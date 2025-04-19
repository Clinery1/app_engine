#version 460 core

layout(binding = 0) uniform sampler2D image;
layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 vk_color;

void main() {
    vk_color = texture(image, uv);
    // vk_color = vec4(1.0);
}
