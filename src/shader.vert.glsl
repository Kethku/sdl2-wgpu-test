#version 450

layout(location = 0) in vec2 a_VertexPosition;
layout(location = 1) in vec2 a_Position;
layout(location = 2) in vec2 a_Dimensions;
layout(location = 3) in vec3 a_Color;

layout(location = 0) out vec3 v_Color;

layout(set = 0, binding = 0) uniform Globals {
    vec2 u_FontSize;
    vec2 u_WindowSize;
};

void main() {
    v_Color = a_Color;
    vec2 gridPosition = a_Position +  a_Dimensions * a_VertexPosition;
    gl_Position = vec4((gridPosition * u_FontSize) / (u_WindowSize * 2.0) - vec2(1.0, 1.0), 0.0, 1.0);
}
