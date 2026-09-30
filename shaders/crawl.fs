#version 330
out vec4 finalColor;
uniform sampler2D crawl;
uniform vec2 resolution;
uniform float elapsed;
float hash(vec2 p) {return fract(sin(dot(p,vec2(127.1,311.7)))*43758.5453);}
void main() {
    vec2 uv=gl_FragCoord.xy/resolution;
    vec2 grid=uv*vec2(380.,240.);vec2 cell=floor(grid);
    float seed=hash(cell);
    float star=seed>.988?pow(max(1.-length(fract(grid)-.5)*3.,0.),2.)*(.4+seed*.6):0.;
    vec3 color=vec3(.002,.005,.014)+vec3(.65,.8,1.)*star;
    float horizon=.79;
    if(elapsed>4.2 && uv.y<horizon) {
        float scale=(horizon-uv.y)/horizon;
        float depth=.34*uv.y/(horizon-uv.y);
        vec2 textUV=vec2((uv.x-.5)/(.90*scale)+.5,(elapsed-4.2)*.105-depth);
        if(textUV.x>=0. && textUV.x<=1. && textUV.y>=0. && textUV.y<=1.) {
            vec4 ink=texture(crawl,vec2(textUV.x,1.-textUV.y));
            float fade=1.-smoothstep(.58,.78,uv.y);
            color=mix(color,ink.rgb,ink.a*fade);
        }
    }
    color*=1.-smoothstep(24.,26.,elapsed);
    finalColor=vec4(color,1.);
}
