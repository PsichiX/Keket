/* ANCHOR: use */
use keket::{
    database::{AssetDatabase, handle::AssetDependency, reference::AssetRef},
    fetch::{deferred::DeferredAssetFetch, file::FileAssetFetch},
    protocol::{
        bundle::{BundleAssetProtocol, BundleWithDependencies, BundleWithDependenciesProcessor},
        group::GroupAssetProtocol,
        text::TextAssetProtocol,
    },
    third_party::anput::{
        commands::{CommandBuffer, InsertCommand, RemoveCommand},
        entity::Entity,
        world::{Relation, World},
    },
};
use serde::{Deserialize, Serialize};
use spitfire::prelude::*;
use std::{
    error::Error,
    sync::{Arc, RwLock},
    time::Instant,
};
/* ANCHOR_END: use */

/* ANCHOR: main */
fn main() -> Result<(), Box<dyn Error>> {
    App::<Vertex>::default().run(State::default());

    Ok(())
}
/* ANCHOR_END: main */

const DELTA_TIME: f32 = 1.0 / 60.0;

/* ANCHOR: state_struct */
struct State {
    // We store drawing context for later use in app state.
    // Drawing context holds resources and stack-based states.
    context: DrawContext,
    // Timer used for fixed step frame particle system simulation.
    timer: Instant,
    assets: AssetDatabase,
    image_shader: AssetRef,
    ferris_texture: AssetRef,
}
/* ANCHOR_END: state_struct */

/* ANCHOR: state_impl_default */
impl Default for State {
    fn default() -> Self {
        Self {
            context: Default::default(),
            timer: Instant::now(),
            assets: AssetDatabase::default()
                // Text protocol for shader sources.
                .with_protocol(TextAssetProtocol)
                // Group protocol for loading many assets at once.
                .with_protocol(GroupAssetProtocol)
                // Custom shader protocol.
                .with_protocol(BundleAssetProtocol::new("shader", ShaderAssetProcessor))
                // Custom texture protocol.
                .with_protocol(BundleAssetProtocol::new("texture", TextureAssetProcessor))
                // Load all data from file system asynchronously.
                .with_fetch(DeferredAssetFetch::new(
                    FileAssetFetch::default().with_root("resources"),
                )),
            // Stored asset references for cached asset handles.
            image_shader: AssetRef::new("shader://image.shader"),
            ferris_texture: AssetRef::new("texture://ferris.png"),
        }
    }
}
/* ANCHOR_END: state_impl_default */

/* ANCHOR: state_impl_appstate */
impl AppState<Vertex> for State {
    fn on_init(&mut self, graphics: &mut Graphics<Vertex>, _: &mut AppControl) {
        // Setup scene camera.
        graphics.color = [0.25, 0.25, 0.25, 1.0];
        graphics.main_camera.screen_alignment = 0.5.into();
        graphics.main_camera.scaling = CameraScaling::FitToView {
            size: 1000.0.into(),
            inside: false,
        };

        // Load this scene group.
        self.assets.ensure("group://ingame.txt").unwrap();
    }

    fn on_redraw(&mut self, graphics: &mut Graphics<Vertex>, _: &mut AppControl) {
        // Process assets periotically.
        if self.timer.elapsed().as_secs_f32() > DELTA_TIME {
            self.timer = Instant::now();
            self.process_assets(graphics);
        }

        // Do not render unless we have shader loaded.
        let Ok(image_shader) = self.image_shader.resolve(&self.assets) else {
            return;
        };
        let Some(image_shader) = image_shader
            .access_checked::<&AsyncHandle<Shader>>()
            .map(|handle| handle.to_ref())
        else {
            return;
        };

        // Begin drawing objects.
        self.context.begin_frame(graphics);
        self.context.push_shader(&image_shader);
        self.context.push_blending(GlowBlending::Alpha);

        // Draw sprite only if texture asset is loaded.
        if let Ok(texture) = self.ferris_texture.resolve(&self.assets) {
            if let Some(texture) = texture
                .access_checked::<&AsyncHandle<Texture>>()
                .map(|handle| handle.to_ref())
            {
                Sprite::single(SpriteTexture::new("u_image".into(), texture))
                    .pivot(0.5.into())
                    .draw(&mut self.context, graphics);
            }
        }

        // Commit drawn objects.
        self.context.end_frame();
    }
}
/* ANCHOR_END: state_impl_appstate */

/* ANCHOR: state_impl */
impl State {
    fn process_assets(&mut self, graphics: &mut Graphics<Vertex>) {
        let mut commands = CommandBuffer::default();

        // Process loaded shader assets into shader objects on GPU.
        for entity in self.assets.storage.added().iter_of::<ShaderAsset>() {
            let asset = self
                .assets
                .storage
                .component::<true, ShaderAsset>(entity)
                .unwrap();
            let shader = graphics.shader(&asset.vertex, &asset.fragment).unwrap();
            println!("* Shader asset turned into shader: {entity}");
            commands.command(InsertCommand::new(entity, (AsyncHandle::new(shader),)));
        }

        // Process loaded texture assets into texture objects on GPU.
        for entity in self.assets.storage.added().iter_of::<TextureAsset>() {
            let asset = self
                .assets
                .storage
                .component::<true, TextureAsset>(entity)
                .unwrap();
            let texture = graphics
                .texture(
                    asset.width,
                    asset.height,
                    1,
                    GlowTextureFormat::Rgba,
                    Some(&asset.bytes),
                )
                .unwrap();
            println!("* Texture asset turned into texture: {entity}");
            commands.command(InsertCommand::new(entity, (AsyncHandle::new(texture),)));
        }

        commands.execute(&mut self.assets.storage);
        self.assets.maintain().unwrap();
    }
}
/* ANCHOR_END: state_impl */

/* ANCHOR: async_handle */
// Workaround for GPU handles not being Send + Sync,
// to be able to store them in assets database.
struct AsyncHandle<T: Clone>(Arc<RwLock<T>>);

unsafe impl<T: Clone> Send for AsyncHandle<T> {}
unsafe impl<T: Clone> Sync for AsyncHandle<T> {}

impl<T: Clone> AsyncHandle<T> {
    fn new(data: T) -> Self {
        Self(Arc::new(RwLock::new(data)))
    }

    fn get(&self) -> T {
        self.0.read().unwrap().clone()
    }

    fn to_ref(&self) -> ResourceRef<T> {
        ResourceRef::object(self.get())
    }
}
/* ANCHOR_END: async_handle */

/* ANCHOR: shader_protocol */
// Decoded shader asset information with dependencies.
#[derive(Debug, Serialize, Deserialize)]
struct ShaderAssetInfo {
    vertex: AssetRef,
    fragment: AssetRef,
}

// Shader asset with vertex and fragment programs code.
struct ShaderAsset {
    vertex: String,
    fragment: String,
}

// Shader asset processor that turns bytes -> shader info -> shader asset.
struct ShaderAssetProcessor;

impl BundleWithDependenciesProcessor for ShaderAssetProcessor {
    type Bundle = (ShaderAssetInfo,);

    fn process_bytes(
        &mut self,
        bytes: Vec<u8>,
    ) -> Result<BundleWithDependencies<Self::Bundle>, Box<dyn Error>> {
        let asset = serde_json::from_slice::<ShaderAssetInfo>(&bytes)?;
        let vertex = asset.vertex.path().clone();
        let fragment = asset.fragment.path().clone();

        println!("* Shader asset processed: {asset:#?}");
        Ok(BundleWithDependencies::new((asset,))
            .dependency(vertex)
            .dependency(fragment))
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        let mut commands = CommandBuffer::default();
        let mut lookup = storage.lookup_access::<true, &String>();

        // We scan for decoded shader info and if dependencies are loaded,
        // then turn them into shader asset.
        for (entity, info, dependencies) in
            storage.query::<true, (Entity, &ShaderAssetInfo, &Relation<AssetDependency>)>()
        {
            if dependencies
                .entities()
                .all(|entity| storage.has_entity_component::<String>(entity))
            {
                let vertex = lookup
                    .access(storage.find_by::<true, _>(info.vertex.path()).unwrap())
                    .unwrap()
                    .to_owned();
                let fragment = lookup
                    .access(storage.find_by::<true, _>(info.fragment.path()).unwrap())
                    .unwrap()
                    .to_owned();

                let asset = ShaderAsset { vertex, fragment };
                commands.command(InsertCommand::new(entity, (asset,)));
                commands.command(RemoveCommand::<(ShaderAssetInfo,)>::new(entity));
            }
        }
        drop(lookup);

        commands.execute(storage);

        Ok(())
    }
}
/* ANCHOR_END: shader_protocol */

/* ANCHOR: texture_protocol */
// Decoded texture asset with its size and decoded bitmap bytes.
struct TextureAsset {
    width: u32,
    height: u32,
    bytes: Vec<u8>,
}

struct TextureAssetProcessor;

impl BundleWithDependenciesProcessor for TextureAssetProcessor {
    type Bundle = (TextureAsset,);

    fn process_bytes(
        &mut self,
        bytes: Vec<u8>,
    ) -> Result<BundleWithDependencies<Self::Bundle>, Box<dyn Error>> {
        // Decode PNG image into texture size and bitmap bytes.
        let decoder = png::Decoder::new(bytes.as_slice());
        let mut reader = decoder.read_info()?;
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf)?;
        let bytes = buf[..info.buffer_size()].to_vec();

        println!("* Texture asset processed: {info:#?}");
        Ok(BundleWithDependencies::new((TextureAsset {
            width: info.width,
            height: info.height,
            bytes,
        },)))
    }
}
/* ANCHOR_END: texture_protocol */
