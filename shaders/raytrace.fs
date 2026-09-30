#version 330
#define BVH_STACK_SIZE 64
// Raytracer de bloques: BVH, sombras transparentes y óptica con pila acotada.
in vec2 fragTexCoord;
out vec4 finalColor;
uniform sampler2D sceneData;
uniform sampler2D skyData;
uniform int nodeCount, blockBase, materialBase, lightBase, lightCount, skySize;
uniform int quality, reflections, refractions, skyEnabled, spaceMode;
uniform vec2 resolution;
uniform vec3 eye, forward, right, up, saberBottom, saberTop;
uniform float cameraScale;
vec4 dataAt(int i) { return texelFetch(sceneData, ivec2(i % 1024, i / 1024), 0); }
struct Hit { float t; int object; vec3 p; vec3 n; };
struct Material { vec3 albedo; float spec; float shine; float trans; float refl; float ior; vec3 emission; int tex; };
Material material(int i) {
    vec4 a=dataAt(materialBase+i*3), b=dataAt(materialBase+i*3+1), c=dataAt(materialBase+i*3+2);
    return Material(a.xyz,a.w,b.x,b.y,b.z,b.w,c.xyz,int(c.w));
}
// El tratamiento explícito de direcciones paralelas evita 0*infinito/NaN.
bool interval(vec3 o, vec3 d, vec3 inv, vec3 lo, vec3 hi, float limit, bool conservative, out float nearT, out float farT) {
    bvec3 parallel=lessThan(abs(d),vec3(1e-9));
    if(any(bvec3(parallel.x&&(o.x<lo.x||o.x>hi.x),parallel.y&&(o.y<lo.y||o.y>hi.y),parallel.z&&(o.z<lo.z||o.z>hi.z)))) return false;
    vec3 t0=(lo-o)*inv,t1=(hi-o)*inv;
    vec3 e=conservative?4.7683716e-7*max(max(abs(t0),abs(t1)),vec3(1)):vec3(0);
    vec3 nearV=mix(min(t0,t1)-e,vec3(-1e30),parallel);
    vec3 farV=mix(max(t0,t1)+e,vec3(1e30),parallel);
    nearT=max(max(nearV.x,nearV.y),nearV.z);
    farT=min(min(min(farV.x,farV.y),farV.z),limit);
    return nearT<=farT && farT>0.0005;
}
bool intersectSceneImpl(vec3 o, vec3 d, float limit, bool shadowRay, out Hit hit) {
    vec3 inv=vec3(abs(d.x)<1e-9?0.:1./d.x,abs(d.y)<1e-9?0.:1./d.y,abs(d.z)<1e-9?0.:1./d.z);
    int found=-1; float best=limit;
    int stack[BVH_STACK_SIZE]; float distances[BVH_STACK_SIZE]; int pending=1;
    stack[0]=0; distances[0]=-1e30;
    for(int visited=0;visited<nodeCount && pending>0;visited++) {
        pending--; int node=stack[pending]; if(distances[pending]>best) continue;
        vec4 lo=dataAt(node*3),hi=dataAt(node*3+1),meta=dataAt(node*3+2);
        float tn,tf; int count=int(hi.w);
        if(count==0) {
            int left=int(meta.y),rightNode=int(meta.z); float nearLeft,nearRight,farLeft,farRight;
            bool a=interval(o,d,inv,dataAt(left*3).xyz,dataAt(left*3+1).xyz,best,true,nearLeft,farLeft);
            bool b=interval(o,d,inv,dataAt(rightNode*3).xyz,dataAt(rightNode*3+1).xyz,best,true,nearRight,farRight);
            if(a && b) {
                bool firstLeft=nearLeft<nearRight;
                stack[pending]=firstLeft?rightNode:left; distances[pending++]=firstLeft?nearRight:nearLeft;
                stack[pending]=firstLeft?left:rightNode; distances[pending++]=firstLeft?nearLeft:nearRight;
            } else if(a || b) { stack[pending]=a?left:rightNode; distances[pending++]=a?nearLeft:nearRight; }
            continue;
        }
        int start=int(meta.x);
        for(int j=0;j<count;j++) {
            int b=blockBase+(start+j)*3;
            if(interval(o,d,inv,dataAt(b).xyz,dataAt(b+1).xyz,1e30,false,tn,tf)) {
                float t=tn>0.0005?tn:tf;
                if(t<best) {
                    if(shadowRay && dataAt(b+1).w==0.) { hit=Hit(t,start+j,o+d*t,vec3(0)); return true; }
                    best=t; found=start+j;
                }
            }
        }
    }
    if(found<0) return false;
    vec3 p=o+d*best, lo=dataAt(blockBase+found*3).xyz, hi=dataAt(blockBase+found*3+1).xyz;
    vec3 normal=vec3(0); float closest=1e30;
    for(int a=0;a<3;a++) {
        float delta=abs(p[a]-lo[a]);
        if(delta<closest) { closest=delta; normal=vec3(0); normal[a]=-1.; }
        delta=abs(p[a]-hi[a]);
        if(delta<closest) { closest=delta; normal=vec3(0); normal[a]=1.; }
    }
    hit=Hit(best,found,p,normal); return true;
}
bool intersectScene(vec3 o,vec3 d,float limit,out Hit hit) { return intersectSceneImpl(o,d,limit,false,hit); }
uint hashValue(uint n) {
    n=n*747796405u+2891336453u;
    uint word=((n>>((n>>28u)+4u))^n)*277803737u;
    return (word>>22u)^word;
}
float noiseValue(ivec3 p) { return float(hashValue(uint(p.x)*73856093u ^ uint(p.y)*19349663u ^ uint(p.z)*83492791u))/4294967295.; }
vec3 surfaceColor(Material m,vec3 p,vec3 n) {
    vec2 uv=abs(n.y)>.5?p.xz:(abs(n.x)>.5?p.zy:p.xy);
    float u=uv.x,v=uv.y,grain=noiseValue(ivec3(ivec2(floor(uv*48.)),31));
    float f=1.;
    if(m.tex==0) {
        bool seam=mod(u,.72)<.014 || mod(v,.72)<.014;
        bool bolt=mod(u,.72)<.055 && mod(v,.72)<.055;
        f=seam?.34:(bolt?.25:.72+noiseValue(ivec3(ivec2(floor(uv/.72)),7))*.30+grain*.06);
    } else if(m.tex==1) f=.7+grain*.3+(abs(v*35.-trunc(v*35.))<.2?.2:0.);
    else if(m.tex==2) f=.97+.03*grain;
    else if(m.tex==3) f=.8+.20*noiseValue(ivec3(floor(p.xzy*2.)))+.16*grain;
    else if(m.tex==4) f=mod(u,.18)<.025?.35:.85+.15*grain;
    else if(m.tex==5) f=.82+.12*sin(u*180.)*sin(v*180.)+.06*grain;
    else if(m.tex==6) f=.93+.07*grain;
    else f=.9+.1*abs(sin(v*90.));
    return m.albedo*f;
}
vec3 sky(vec3 d) {
    if(skyEnabled==0) return vec3(.05,.06,.08);
    vec3 a=abs(d); int face; vec2 uv;
    if(a.x>=a.y && a.x>=a.z) { face=d.x>=0.?0:1; uv=vec2(d.x>=0.?-d.z:d.z,d.y)/a.x; }
    else if(a.y>=a.z) { face=d.y>=0.?2:3; uv=vec2(d.x,d.y>=0.?-d.z:d.z)/a.y; }
    else { face=d.z>=0.?4:5; uv=vec2(d.z>=0.?d.x:-d.x,d.y)/a.z; }
    vec2 p=clamp((uv+1.)*.5*float(skySize-1),vec2(0),vec2(skySize-1));
    ivec2 p0=ivec2(floor(p)),p1=min(p0+1,ivec2(skySize-1));
    int y=face*skySize;
    vec3 a0=texelFetch(skyData,ivec2(p0.x,p0.y+y),0).rgb,b0=texelFetch(skyData,ivec2(p1.x,p0.y+y),0).rgb;
    vec3 a1=texelFetch(skyData,ivec2(p0.x,p1.y+y),0).rgb,b1=texelFetch(skyData,ivec2(p1.x,p1.y+y),0).rgb;
    return mix(mix(a0,b0,fract(p.x)),mix(a1,b1,fract(p.x)),fract(p.y));
}
vec3 shadow(vec3 o,vec3 light) {
    vec3 delta=light-o,d=normalize(delta),visibility=vec3(1); float distance=length(delta);
    for(int i=0;i<10;i++) {
        Hit h; if(!intersectSceneImpl(o,d,distance,true,h)) return visibility;
        Material m=material(int(dataAt(blockBase+h.object*3).w));
        if(m.trans<=0.) return vec3(0);
        visibility*=mix(m.albedo,vec3(1),.8)*sqrt(m.trans);
        distance-=h.t+.002; o=h.p+d*.002;
    }
    return vec3(0);
}
vec3 saberGlow(vec3 o,vec3 d,float limit) {
    vec3 v=saberTop-saberBottom,w=o-saberBottom;
    float b=dot(d,v),c=dot(v,v),dw=dot(d,w),e=dot(v,w);
    float t=c-b*b>1e-6?clamp((e-b*dw)/(c-b*b),0.,1.):clamp(e/c,0.,1.);
    vec3 point=saberBottom+v*t;float along=dot(point-o,d);
    if(along<0. || along>limit) return vec3(0);
    vec3 delta=o+d*along-point;float distance=dot(delta,delta);
    return vec3(1.,.003,.001)*(exp(-distance/.009)*1.8+exp(-distance/.045)*.12);
}
struct Task { vec3 o; vec3 d; vec3 throughput; float weight; int depth; };
vec3 traceRay(vec3 origin, vec3 direction) {
    Task tasks[8]; int count=1;
    tasks[0]=Task(origin,direction,vec3(1),1.,0);
    vec3 result=vec3(0); int maxDepth=quality==0?3:6;
    // Un árbol binario de profundidad 6 contiene como máximo 127 tareas.
    for(int step=0;step<127 && count>0;step++) {
        Task task=tasks[--count]; Hit hit;
        if(!intersectScene(task.o,task.d,1e30,hit)) { result+=task.throughput*(sky(task.d)+saberGlow(task.o,task.d,1e30)); continue; }
        result+=task.throughput*saberGlow(task.o,task.d,hit.t);
        int b=blockBase+hit.object*3;
        Material mat=material(int(dataAt(b).w)); vec3 tint=dataAt(b+2).xyz;
        bool front=dot(task.d,hit.n)<0.; vec3 n=front?hit.n:-hit.n;
        vec3 surface=surfaceColor(mat,hit.p,hit.n)*tint;
        vec3 diffuse=spaceMode!=0?vec3(.055,.07,.115):vec3(.095,.12,.17), specular=vec3(0);
        if(task.depth==0 && quality>0 && mat.trans==0.) {
            vec3 tangent=normalize(cross(n,abs(n.y)<.9?vec3(0,1,0):vec3(1,0,0))),bitangent=cross(n,tangent);
            float occlusion=0.;
            for(int a=0;a<2;a++) {
                vec3 d=normalize(n*.85+(a==0?1.:-1.)*(tangent*.4+bitangent*.3)); Hit h;
                if(intersectScene(hit.p+n*.002,d,.65,h)) occlusion+=1.-h.t/.65;
            }
            diffuse*=1.-occlusion*.28;
        }
        for(int i=0;i<lightCount;i++) {
            vec4 pos=dataAt(lightBase+i*2),color=dataAt(lightBase+i*2+1);
            float attenuation=color.w>0.?pow(max(1.-length(pos.xyz-hit.p)/color.w,0.),2.):1.;
            float intensity=pos.w*attenuation;
            if(spaceMode!=0 && i<2) {intensity*=.60;color.xyz=mix(color.xyz,vec3(.48,.65,1.),.65); }
            if(intensity<.001) continue;
            vec3 l=normalize(pos.xyz-hit.p); float ndotl=max(dot(n,l),0.);
            if(ndotl<=0.) continue;
            vec3 visibility=shadow(hit.p+n*.002,pos.xyz);
            diffuse+=color.xyz*visibility*(intensity*ndotl);
            float highlight=pow(max(dot(n,normalize(l-task.d)),0.),mat.shine)*mat.spec;
            specular+=color.xyz*visibility*(highlight*intensity);
        }
        float reflected=reflections!=0?mat.refl:0.,transparent=refractions!=0?mat.trans:0.;
        float cosI=clamp(-dot(task.d,n),0.,1.),r0=pow((1.-mat.ior)/(1.+mat.ior),2.);
        float fresnel=r0+(1.-r0)*pow(1.-cosI,5.);
        reflected+=reflections!=0?transparent*fresnel:0.;
        float transmitted=transparent*(1.-(reflections!=0?fresnel:0.));
        vec3 refracted=refract(task.d,n,front?1./mat.ior:mat.ior);
        if(dot(refracted,refracted)<.5) { reflected+=transmitted; transmitted=0.; }
        result+=task.throughput*((surface*diffuse+specular)*max(1.-reflected-transmitted,0.)+mat.emission*tint);
        if(reflected>0.) {
            vec3 d=normalize(reflect(task.d,n)),throughput=task.throughput*reflected;
            if(task.depth<maxDepth && task.weight*reflected>.012)
                tasks[count++]=Task(hit.p+n*.002,d,throughput,task.weight*reflected,task.depth+1);
            else result+=throughput*sky(d);
        }
        if(transmitted>0.) {
            vec3 d=normalize(refracted),throughput=task.throughput*mix(surface,vec3(1),.92)*transmitted;
            if(task.depth<maxDepth && task.weight*transmitted>.012)
                tasks[count++]=Task(hit.p-n*.002,d,throughput,task.weight*transmitted,task.depth+1);
            else result+=throughput*sky(d);
        }
    }
    return result;
}
void main() {
    // gl_FragCoord pertenece al framebuffer físico, independiente del DPI de la ventana.
    vec2 pixel=vec2(gl_FragCoord.x,resolution.y-gl_FragCoord.y);
    vec3 color=vec3(0); int samples=quality==2?4:1;
    for(int i=0;i<samples;i++) {
        vec2 offset=samples==1?vec2(0):vec2((i%2==0)?-.25:.25,(i<2)?-.25:.25);
        vec2 uv=(pixel+offset)/resolution;
        vec3 d=normalize(forward+right*((2.*uv.x-1.)*resolution.x/resolution.y*cameraScale)+up*((1.-2.*uv.y)*cameraScale));
        color+=traceRay(eye,d);
    }
    color=max(color/float(samples),vec3(0))*.90;
    color=clamp(color*(2.51*color+.03)/(color*(2.43*color+.59)+.14),0.,1.);
    finalColor=vec4(pow(color,vec3(1./2.2)),1.);
}
