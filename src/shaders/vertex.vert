#version 450

layout (push_constant) uniform Uniforms {
    float invAspect;
    float renderScale;
    float renderOffsX, renderOffsZ;
    float starDensity;
} uniforms;

layout (location = 0) out vec3 out_vColor;

// per vertex attributes
layout (location = 0) in vec2 in_vPos;

// per instance attributes
layout (location = 1) in vec2 in_iPos;
layout (location = 2) in vec3 in_iColor;
layout (location = 3) in float in_iRadius;

void main() {
    out_vColor = in_iColor;

    vec2 position = vec2(uniforms.renderOffsX, uniforms.renderOffsZ) + in_vPos * in_iRadius;
    gl_Position = vec4(vec3((position + in_iPos) * vec2(uniforms.invAspect, 1.0) * uniforms.renderScale, 0.0), 1.0);
}
