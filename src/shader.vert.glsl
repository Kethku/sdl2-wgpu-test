#version 450

layout(location = 0) in vec2 a_Position;
layout(location = 1) in vec2 a_Dimensions;
layout(location = 2) in vec2 a_VertexPosition;

layout(set = 0, binding = 0) uniform Globals {
    vec2 u_GridDimensions;
    vec2 u_FontSize;
};

void main() {
    gl_Position = vec4(a_Position * a_Dimensions * a_VertexPosition / (u_GridDimensions * u_FontSize), 0, 0);
    gl_Position.x = -gl_Position.x;
    gl_Position.y = -gl_Position.y;
}
