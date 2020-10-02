#version 460 core

out vec4 fc;

uniform float b;

void main() {
    fc = vec4(0, 0, b, 1.0);
}