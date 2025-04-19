#version 460 core

layout(push_constant) uniform pc {
    mat3 transform;
};

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv_in;

layout(location = 0) out vec2 uv;

void main() {
    gl_Position = vec4(transform * vec3(position, 1.0), 1);
    uv = uv_in;
}
