#version 300 es

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec3 a_uv;
layout(location = 2) in vec4 a_color;

out vec4 v_color;
out vec3 v_uv;
uniform mat4 u_projection_view;

void main() {
    gl_Position = u_projection_view * vec4(a_position, 0.0, 1.0);
    v_color = a_color;
    v_uv = a_uv;
}
