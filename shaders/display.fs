#version 330
// Suavizado de bordes en pantalla. No añade rayos ni altera materiales o iluminación.
in vec2 fragTexCoord;
in vec4 fragColor;
out vec4 finalColor;
uniform sampler2D texture0;
uniform vec2 texelSize;
float luma(vec3 c) { return dot(c,vec3(.299,.587,.114)); }
void main() {
    vec2 uv=fragTexCoord;
    vec3 center=texture(texture0,uv).rgb;
    vec3 nw=texture(texture0,uv+vec2(-1,-1)*texelSize).rgb;
    vec3 ne=texture(texture0,uv+vec2(1,-1)*texelSize).rgb;
    vec3 sw=texture(texture0,uv+vec2(-1,1)*texelSize).rgb;
    vec3 se=texture(texture0,uv+vec2(1,1)*texelSize).rgb;
    float m=luma(center),a=luma(nw),b=luma(ne),c=luma(sw),d=luma(se);
    float low=min(m,min(min(a,b),min(c,d))),high=max(m,max(max(a,b),max(c,d)));
    if(high-low<max(.045,high*.125)) { finalColor=vec4(center,1)*fragColor; return; }
    vec2 dir=vec2(-((a+b)-(c+d)),(a+c)-(b+d));
    float reduce=max((a+b+c+d)*(.25*.125),1./128.);
    float reciprocal=1./(min(abs(dir.x),abs(dir.y))+reduce);
    dir=clamp(dir*reciprocal,vec2(-6),vec2(6))*texelSize;
    vec3 inner=.5*(texture(texture0,uv-dir/6.).rgb+texture(texture0,uv+dir/6.).rgb);
    vec3 outer=inner*.5+.25*(texture(texture0,uv-dir*.5).rgb+texture(texture0,uv+dir*.5).rgb);
    float l=luma(outer);
    finalColor=vec4(l<low||l>high?inner:outer,1)*fragColor;
}
