<template>
  <canvas ref="pane" class="hole-layer" :class="{ 'hole-layer--live': ready }"></canvas>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';

const props = defineProps<{ active: boolean; detail?: string }>();

const pane = ref<HTMLCanvasElement | null>(null);
const ready = ref(false);

let gl: WebGLRenderingContext | null = null;
let sceneProg: WebGLProgram | null = null;
let postProg:  WebGLProgram | null = null;
let downProg: WebGLProgram | null = null;
let upProg:   WebGLProgram | null = null;
type Target = { tex: WebGLTexture; fbo: WebGLFramebuffer; w: number; h: number };
let film: Target | null = null;
let downs: Target[] = [];
let ups: Target[] = [];
let texType = 0;
let meshBuf: WebGLBuffer | null = null;
let frameId = 0;
let born = 0;
let dozing = false;
let aimX = 0, aimY = 0, driftX = 0, driftY = 0;

const vertSrc = `attribute vec2 spot;
varying vec2 vUv;
void main(){vUv=spot*0.5+0.5;gl_Position=vec4(spot,0.0,1.0);}`;

const sceneSrc    = `precision highp float;
uniform vec2 res;
uniform float time;
uniform vec2 drift;

const float DIN = 2.6;
const float DOUT = 9.5;

float hash(vec2 s){return fract(sin(dot(s,vec2(127.1,311.7)))*43758.5453123);}
float noise(vec2 s){
  vec2 i=floor(s);vec2 f=fract(s);vec2 u=f*f*(3.0-2.0*f);
  float a=hash(i);float b=hash(i+vec2(1.0,0.0));float c=hash(i+vec2(0.0,1.0));float d=hash(i+vec2(1.0,1.0));
  return mix(mix(a,b,u.x),mix(c,d,u.x),u.y);
}
float fbm(vec2 s){
  float acc=0.0;float amp=0.5;
  for(int i=0;i<4;i++){acc+=amp*noise(s);s=s*2.03+vec2(19.7,7.3);amp*=0.5;}
  return acc;
}
mat2 whirl(float a){float c=cos(a);float s=sin(a);return mat2(c,-s,s,c);}
vec3 bbody(float k){
  vec3 col=mix(vec3(0.45,0.08,0.02),vec3(1.0,0.42,0.12),smoothstep(0.0,0.4,k));
  col=mix(col,vec3(1.0,0.85,0.62),smoothstep(0.4,0.75,k));
  col=mix(col,vec3(0.85,0.90,1.0),smoothstep(0.75,1.05,k));
  return col;
}
float diskGlow(vec3 hit,float rr){
  float ang=atan(hit.z,hit.x);
  float omega=6.0/pow(rr,1.5);
  float shear=ang+(fract(time/36.0)-0.5)*36.0*omega;
  float lr=log(rr);
  float lanes=noise(vec2(shear*4.0,lr*5.0))*0.65+noise(vec2(shear*8.0+47.3,lr*11.0+9.1))*0.35;
  float calm=smoothstep(DOUT*0.9,DOUT*0.4,rr);
  lanes=mix(0.5,lanes,0.3+0.7*calm);
  float fadeIn=smoothstep(DIN,DIN*1.35,rr);
  float fadeOut=1.0-smoothstep(DOUT*0.45,DOUT*0.95,rr);
  return (0.35+0.9*lanes)*fadeIn*fadeOut;
}
float hash3(vec3 s){return fract(sin(dot(s,vec3(127.1,311.7,74.7)))*43758.5453123);}
float noise3(vec3 s){
  vec3 i=floor(s);vec3 f=fract(s);
  f=f*f*(3.0-2.0*f);
  float a=hash3(i);float b=hash3(i+vec3(1.0,0.0,0.0));
  float c=hash3(i+vec3(0.0,1.0,0.0));float d=hash3(i+vec3(1.0,1.0,0.0));
  float e=hash3(i+vec3(0.0,0.0,1.0));float g=hash3(i+vec3(1.0,0.0,1.0));
  float h=hash3(i+vec3(0.0,1.0,1.0));float k=hash3(i+vec3(1.0,1.0,1.0));
  return mix(mix(mix(a,b,f.x),mix(c,d,f.x),f.y),mix(mix(e,g,f.x),mix(h,k,f.x),f.y),f.z);
}
float fbm3(vec3 s){
  float acc=0.0;float amp=0.5;
  for(int i=0;i<5;i++){acc+=amp*noise3(s);s=s*2.03+vec3(19.7,7.3,11.1);amp*=0.5;}
  return acc;
}
float starfield(vec3 dir,float cells,float cut,float sd){
  vec3 cell=floor(dir*cells)+0.5;
  if(hash3(cell+sd)<cut){return 0.0;}
  vec3 jitter=vec3(hash3(cell+sd+7.3),hash3(cell+sd+3.1),hash3(cell+sd+1.7))-0.5;
  float ang=max(dot(normalize(cell+jitter),dir),0.0);
  return exp(-(1.0-ang)*480.0*cells*cells);
}
float stars(vec2 q,float cells,float cut){
  vec2 cell=floor(q*cells);
  float seed=hash(cell);
  if(seed<cut){return 0.0;}
  vec2 pos=fract(q*cells)-vec2(hash(cell+7.3),hash(cell+3.1));
  return exp(-dot(pos,pos)*240.0);
}
void main(){
  vec2 sc=(gl_FragCoord.xy-res*vec2(0.80,0.56))/res.y;
  sc=whirl(-0.22)*sc;
  float aim=length(sc);
  float yaw=0.18*sin(time*0.021)+drift.x*0.10;
  float pitch=0.30+0.05*sin(time*0.013+1.7)+drift.y*0.06;
  vec3 ro=vec3(sin(yaw)*cos(pitch),sin(pitch),cos(yaw)*cos(pitch))*24.0;
  vec3 fwd=normalize(-ro);
  vec3 right=normalize(cross(vec3(0.0,1.0,0.0),fwd));
  vec3 up=cross(fwd,right);
  vec3 rd=normalize(fwd*1.5+right*sc.x+up*sc.y);
  vec3 p=ro;
  vec3 v=rd;
  vec3 hv=cross(p,v);
  float h2=dot(hv,hv);
  vec3 acc=vec3(0.0);
  float trans=1.0;
  float captured=0.0;
  int hits=0;
  for(int i=0;i<100;i++){
    float r2=dot(p,p);
    if(r2<1.0){captured=1.0;break;}
    if(r2>1600.0){break;}
    float r=sqrt(r2);
    float dt=clamp(0.045*r,0.03,0.6);
    dt=min(dt,abs(p.y)*0.9+0.05);
    vec3 pull=-1.5*h2*p/(r2*r2*r);
    vec3 pPrev=p;
    v+=pull*dt;
    p+=v*dt;
    if(pPrev.y*p.y<0.0&&hits<3){
      float f=pPrev.y/(pPrev.y-p.y);
      vec3 hit=mix(pPrev,p,f);
      float rr=length(hit.xz);
      if(rr>DIN&&rr<DOUT){
        bool secondary=hits>0;
        if(!secondary||rr<DOUT*0.75){
          float glow=diskGlow(hit,rr);
          float temp=pow(DIN/rr,0.75);
          vec3 tint=bbody(temp);
          vec3 tangent=normalize(vec3(-hit.z,0.0,hit.x));
          float beta=min(sqrt(0.5/rr),0.7);
          float dop=1.0/max(1.0-beta*dot(tangent,normalize(v)),0.35);
          float beam=clamp(dop*dop*dop,0.15,3.2);
          float gred=sqrt(max(1.0-1.0/rr,0.0));
          float w=secondary?0.42:1.0;
          acc+=tint*glow*beam*gred*1.6*trans*w;
          trans*=mix(1.0,0.55,clamp(glow,0.0,1.0));
        }
      }
      hits++;
    }
  }
  vec3 col=acc;
  if(captured<0.5){
    vec3 dome=normalize(v);
    vec3 neb=vec3(0.35,0.25,0.55)*fbm3(dome*2.6)+vec3(0.15,0.22,0.45)*fbm3(dome*1.3);
    float glint=starfield(dome,46.0,0.993,0.0)+starfield(dome,130.0,0.996,31.7)*0.7;
    float skyReach=1.0-smoothstep(0.18,0.46,aim);
    col+=(neb*0.4+vec3(0.9,0.93,1.0)*glint*0.8)*skyReach*trans;
    vec3 nebFar=vec3(0.30,0.24,0.50)*fbm3(dome*2.4+5.0)+vec3(0.13,0.20,0.40)*fbm3(dome*1.2+9.4);
    float glintFar=starfield(dome,96.0,0.9935,11.3)+starfield(dome,224.0,0.996,57.9)*0.7;
    col+=(nebFar*0.65+vec3(0.9,0.93,1.0)*glintFar*1.1)*(1.0-skyReach)*trans;
  }
  col=1.0-exp(-col);
  float vin=0.60+0.40*(1.0-smoothstep(0.62,1.6,aim));
  col*=vin;
  float lum=dot(col,vec3(0.299,0.587,0.114));
  float alpha=clamp(lum*1.7+captured*0.9*(1.0-smoothstep(0.75,1.5,aim)),0.0,1.0);
  gl_FragColor=vec4(col,alpha);
}
`;
const sceneDetSrc = `precision highp float;
uniform vec2 res;
uniform float time;
uniform vec2 drift;

const float DIN=2.6;
const float DOUT=12.0;

mat2 rot2(float a){float c=cos(a),s=sin(a);return mat2(c,-s,s,c);}
float hash(vec2 s){return fract(sin(dot(s,vec2(127.1,311.7)))*43758.5453);}
float noise(vec2 p){
  vec2 i=floor(p),f=fract(p);
  f=f*f*(3.0-2.0*f);
  float a=hash(i),b=hash(i+vec2(1.0,0.0)),c=hash(i+vec2(0.0,1.0)),d=hash(i+vec2(1.0,1.0));
  return mix(mix(a,b,f.x),mix(c,d,f.x),f.y);
}
float fbm(vec2 s){float v=0.0,a=0.5;for(int i=0;i<6;i++){v+=a*noise(s);s=s*2.03+vec2(19.7,7.3);a*=0.5;}return v;}

vec3 bbody(float k){
  vec3 c=mix(vec3(0.10,0.04,0.012),vec3(0.45,0.18,0.05),smoothstep(0.0,0.3,k));
  c=mix(c,vec3(0.92,0.50,0.14),smoothstep(0.3,0.58,k));
  c=mix(c,vec3(1.15,0.84,0.42),smoothstep(0.58,0.85,k));
  c=mix(c,vec3(1.30,1.14,0.82),smoothstep(0.85,1.2,k));
  return c;
}

float diskH(float r){return 0.10+0.42*smoothstep(DIN,DOUT,r);}

vec3 diskField(vec3 pos,float t){
  float r=length(pos.xz);
  float ang=atan(pos.z,pos.x);
  float lr=log(r);
  float phase=ang+t*0.55/pow(r,1.5);
  float n1=fbm(vec2(lr*6.5,phase*2.1));
  float n2=fbm(vec2(lr*15.0+13.7,phase*3.4+5.0));
  float n3=fbm(vec2(lr*30.0+7.1,phase*5.5+2.0));
  float fil=n1*0.5+n2*0.32+n3*0.18;
  float env=smoothstep(DIN*0.98,DIN*1.3,r)*(1.0-smoothstep(DOUT*0.55,DOUT,r));
  float dens=smoothstep(0.30,0.74,fil)*env;
  float dl=fbm(vec2(lr*9.0+51.0,phase*2.8+9.0))*0.72+n2*0.28;
  float dust=smoothstep(0.58,0.86,dl)*smoothstep(DIN*1.25,DIN*2.2,r)*(1.0-smoothstep(DOUT*0.6,DOUT,r));
  float hprof=exp(-pow(pos.y/diskH(r),2.0)*3.0);
  return vec3(dens*hprof,dust*hprof,fil);
}

void main(){
  vec2 sc=(gl_FragCoord.xy - res*vec2(0.80,0.56))/min(res.x,res.y);
  sc=rot2(-0.22)*sc;
  float t=time;
  float sway=drift.x,nod=drift.y;

  float dist=25.0;
  float yaw=0.14*sin(t*0.019)+sway*0.07;
  float pitch=0.21+0.035*sin(t*0.011+1.5)+nod*0.045;

  vec3 ro=vec3(dist*cos(pitch)*sin(yaw),dist*sin(pitch),-dist*cos(pitch)*cos(yaw));
  vec3 fw=normalize(-ro);
  vec3 rt=normalize(cross(fw,vec3(0.0,1.0,0.0)));
  vec3 up=cross(rt,fw);
  vec3 rd=normalize(fw*1.75+rt*sc.x+up*sc.y);

  vec3 p=ro;vec3 v=rd;
  vec3 hv=cross(p,v);float h2=dot(hv,hv);

  vec3 acc=vec3(0.0);
  float trans=1.0;
  float captured=0.0;
  float minR=1e4;
  float prevY=p.y;
  float orderW=1.0;
  float winds=0.0;

  for(int i=0;i<300;i++){
    float r2=dot(p,p);
    if(r2<1.0){captured=1.0;break;}
    if(r2>2500.0)break;
    float r=sqrt(r2);
    minR=min(minR,r);
    float rrPre=length(p.xz);
    float dt=0.045*r;
    if(abs(p.y)<diskH(rrPre)+0.35&&rrPre<DOUT+1.0)dt=min(dt,0.06);
    dt=min(dt,0.55);
    v+=-1.5*h2*p/(r2*r2*r)*dt;
    p+=v*dt;

    if(prevY*p.y<0.0){
      winds+=1.0;
      if(winds>6.0)break;
      if(winds>2.0)orderW*=0.55;
    }
    prevY=p.y;

    float rr=length(p.xz);
    if(trans>0.02&&rr>DIN*0.95&&rr<DOUT&&abs(p.y)<diskH(rr)+0.05){
      vec3 df=diskField(p,t);
      float dens=df.x,dust=df.y;
      if(dens+dust>0.003){
        float dop=dot(normalize(vec3(-p.z,0.0,p.x)),normalize(v));
        float beta=0.55/sqrt(max(rr,1.6));
        float dl=1.0/(1.0-beta*dop);
        float beam=pow(clamp(dl,0.45,2.2),3.0)*0.42;
        float gred=sqrt(max(1.0-1.0/max(rr,1.05),0.05));
        float k=clamp(pow(1.0-(rr-DIN)/(DOUT-DIN),1.4),0.0,1.0);
        k=k*(0.55+0.5*df.z)+0.25*(dl-1.0);
        float hot=1.0+3.0*exp(-(rr-DIN)*1.1);
        vec3 tint=bbody(clamp(k,0.0,1.25));
        trans*=exp(-(dens*2.8+dust*5.5)*dt);
        acc+=tint*dens*beam*gred*hot*trans*dt*5.2*orderW;
        acc+=vec3(0.32,0.12,0.045)*dust*beam*0.35*trans*dt*5.2*orderW;
      }
    }
  }

  vec3 col=acc;
  float aim=length(sc);

  float ring=exp(-pow((minR-1.5)*5.0,2.0))*(1.0-captured);
  col+=vec3(1.18,0.92,0.52)*ring*0.9;

  if(captured<0.5){
    vec3 d=normalize(v);
    vec2 sph=vec2(atan(d.z,d.x)*3.2,asin(clamp(d.y,-1.0,1.0))*6.4);
    float st=0.0;
    for(int L=0;L<2;L++){
      vec2 g=sph*(18.0+22.0*float(L))+vec2(7.7*float(L),3.1);
      vec2 cid=floor(g),cf=fract(g);
      float hs=hash(cid);
      vec2 spos=vec2(hash(cid+11.0),hash(cid+37.0));
      float dstar=length(cf-spos);
      float bri=smoothstep(0.92,1.0,hs);
      st+=bri*exp(-dstar*dstar*90.0)*(0.5+0.5*sin(t*(0.6+hs)+hs*31.0));
    }
    float chrom=fbm(sph*1.7)-0.5;
    vec3 stc=mix(vec3(0.80,0.90,1.15),vec3(1.10,0.92,0.80),chrom+0.5);
    col+=stc*st*0.85*trans;
    col+=vec3(0.10,0.06,0.05)*fbm(sph*2.3+4.0)*0.20*trans;
  }

  float ux=gl_FragCoord.x/res.x;
  float uy=gl_FragCoord.y/res.y;
  float mist=fbm(vec2(ux*3.2+t*0.008,uy*2.4-t*0.004));
  float glow=exp(-uy*3.6)*(0.5+0.5*mist);
  col+=vec3(0.05,0.16,0.26)*glow*1.15;

  float lum=dot(col,vec3(0.299,0.587,0.114));
  float alpha=clamp(lum*1.5+captured*0.9*(1.0-smoothstep(0.75,1.5,aim))+glow*0.55,0.0,1.0);
  gl_FragColor=vec4(col,alpha);
}
`;
const postSrc = `precision highp float;
uniform sampler2D tex;
uniform sampler2D bloom;
uniform vec2 res;
uniform float time;
uniform float detail;
varying vec2 vUv;

float hash(vec2 s){return fract(sin(dot(s,vec2(127.1,311.7)))*43758.5453);}
vec3 aces(vec3 x){return clamp((x*(2.51*x+0.03))/(x*(2.43*x+0.59)+0.14),0.0,1.0);}

void main(){
  vec4 base=texture2D(tex,vUv);
  vec2 bpx=1.5/res;
  vec3 bl=texture2D(bloom,vUv).rgb*0.4;
  bl+=texture2D(bloom,vUv+vec2(bpx.x,bpx.y)).rgb*0.15;
  bl+=texture2D(bloom,vUv+vec2(-bpx.x,bpx.y)).rgb*0.15;
  bl+=texture2D(bloom,vUv+vec2(bpx.x,-bpx.y)).rgb*0.15;
  bl+=texture2D(bloom,vUv+vec2(-bpx.x,-bpx.y)).rgb*0.15;
  vec3 col=base.rgb+bl*(1.05+0.55*detail)*vec3(1.12,0.95,0.72);
  col=aces(col);
  float vig=1.0-smoothstep(0.42,1.55,length(vUv-vec2(0.62,0.52))*1.32);
  float sweep=0.32+0.68*smoothstep(-0.14,0.60,vUv.x);
  float shade=(0.52+0.48*vig)*sweep;
  col*=shade;
  col+=(hash(gl_FragCoord.xy+fract(time*0.25)*61.7)-0.5)*0.010;
  float alpha=clamp(base.a*1.05+dot(bl,vec3(0.299,0.587,0.114))*1.4,0.0,1.0);
  alpha*=shade;
  gl_FragColor=vec4(col*alpha,alpha);
}
`;

const sceneNewSrc = `precision highp float;
uniform vec2 res;
uniform float time;
uniform vec2 drift;

float hash(vec2 s){return fract(sin(dot(s,vec2(127.1,311.7)))*43758.5453123);}
float hash3(vec3 s){return fract(sin(dot(s,vec3(127.1,311.7,74.7)))*43758.5453123);}
float noise(vec2 s){
  vec2 i=floor(s);vec2 f=fract(s);vec2 u=f*f*(3.0-2.0*f);
  return mix(mix(hash(i),hash(i+vec2(1.0,0.0)),u.x),mix(hash(i+vec2(0.0,1.0)),hash(i+vec2(1.0,1.0)),u.x),u.y);
}
float noise3(vec3 s){
  vec3 i=floor(s);vec3 f=fract(s);vec3 u=f*f*(3.0-2.0*f);
  float a=mix(hash3(i),hash3(i+vec3(1.0,0.0,0.0)),u.x);
  float b=mix(hash3(i+vec3(0.0,1.0,0.0)),hash3(i+vec3(1.0,1.0,0.0)),u.x);
  float c=mix(hash3(i+vec3(0.0,0.0,1.0)),hash3(i+vec3(1.0,0.0,1.0)),u.x);
  float d=mix(hash3(i+vec3(0.0,1.0,1.0)),hash3(i+vec3(1.0,1.0,1.0)),u.x);
  return mix(mix(a,b,u.y),mix(c,d,u.y),u.z);
}
float fbm(vec2 s){
  float acc=0.0;float amp=0.5;
  for(int i=0;i<4;i++){acc+=amp*noise(s);s=s*2.07+vec2(13.7,5.3);amp*=0.5;}
  return acc;
}
float hash3(vec3 s){return fract(sin(dot(s,vec3(127.1,311.7,74.7)))*43758.5453123);}
float noise3(vec3 s){
  vec3 i=floor(s);vec3 f=fract(s);
  f=f*f*(3.0-2.0*f);
  float a=hash3(i);float b=hash3(i+vec3(1.0,0.0,0.0));
  float c=hash3(i+vec3(0.0,1.0,0.0));float d=hash3(i+vec3(1.0,1.0,0.0));
  float e=hash3(i+vec3(0.0,0.0,1.0));float g=hash3(i+vec3(1.0,0.0,1.0));
  float h=hash3(i+vec3(0.0,1.0,1.0));float k=hash3(i+vec3(1.0,1.0,1.0));
  return mix(mix(mix(a,b,f.x),mix(c,d,f.x),f.y),mix(mix(e,g,f.x),mix(h,k,f.x),f.y),f.z);
}
float fbm3(vec3 s){
  float acc=0.0;float amp=0.5;
  for(int i=0;i<5;i++){acc+=amp*noise3(s);s=s*2.03+vec3(19.7,7.3,11.1);amp*=0.5;}
  return acc;
}
float starfield(vec3 dir,float cells,float cut,float sd){
  vec3 cell=floor(dir*cells)+0.5;
  if(hash3(cell+sd)<cut){return 0.0;}
  vec3 jitter=vec3(hash3(cell+sd+7.3),hash3(cell+sd+3.1),hash3(cell+sd+1.7))-0.5;
  float ang=max(dot(normalize(cell+jitter),dir),0.0);
  return exp(-(1.0-ang)*480.0*cells*cells);
}
float stars(vec2 q,float cells,float cut){
  vec2 cell=floor(q*cells);
  float seed=hash(cell);
  if(seed<cut)return 0.0;
  vec2 pos=fract(q*cells)-vec2(hash(cell+7.3),hash(cell+3.1));
  return exp(-dot(pos,pos)*240.0);
}

const float DIN=2.35;
const float DOUT=11.0;

vec3 bb(float t){
  vec3 c=mix(vec3(0.32,0.05,0.02),vec3(1.05,0.32,0.08),smoothstep(0.0,0.45,t));
  c=mix(c,vec3(1.25,0.78,0.42),smoothstep(0.45,0.78,t));
  c=mix(c,vec3(1.35,1.18,1.02),smoothstep(0.78,1.0,t));
  return c;
}

float streaks(vec3 hp,float rr){
  float om=0.55/pow(rr,1.5);
  float a=atan(hp.z,hp.x)-time*om;
  vec3 q=vec3(cos(a)*1.6,sin(a)*1.6,rr*0.85);
  float n=noise3(q*2.0)*0.55+noise3(q*4.0+vec3(9.7))*0.30+noise3(q*8.0+vec3(23.1))*0.15;
  return n*n*1.6;
}

float diskShine(vec3 hp,float rr){
  float f=smoothstep(DIN,DIN*1.22,rr)*(1.0-smoothstep(DOUT*0.42,DOUT*0.95,rr));
  if(f<=0.0)return 0.0;
  return f*(0.45+0.95*streaks(hp,rr));
}

void main(){
  vec2 uv=(gl_FragCoord.xy/res-vec2(0.71,0.54))*vec2(res.x/res.y,1.0);
  uv+=drift*0.05;
  float aim=length(uv);

  float yaw=0.16*sin(time*0.017);
  float pitch=0.34+0.045*sin(time*0.011+1.3);
  float cy=cos(yaw);float sy=sin(yaw);float cp=cos(pitch);float sp=sin(pitch);
  vec3 ro=vec3(sy*cp,sp,-cy*cp)*26.0;
  vec3 fw=normalize(-ro);
  vec3 rt=normalize(cross(vec3(0.0,1.0,0.0),fw));
  vec3 up=cross(fw,rt);
  vec3 v=normalize(fw*1.6+rt*uv.x+up*uv.y);
  vec3 p=ro;

  vec3 acc=vec3(0.0);
  float trans=1.0;
  float captured=0.0;

  for(int i=0;i<180;i++){
    float r2=dot(p,p);
    float r=sqrt(r2);
    if(r2>2100.0)break;
    if(r<1.0){captured=1.0;break;}
    float dt=clamp(0.05*r,0.035,0.55);
    dt=min(dt,abs(p.y)*0.8+0.05);
    vec3 h=cross(p,v);
    float h2=dot(h,h);
    v+=-1.5*h2*p/(r2*r2*r)*dt;
    vec3 np=p+v*dt;
    if(p.y*np.y<0.0){
      float f=p.y/(p.y-np.y);
      vec3 hp=mix(p,np,f);
      float rr=length(hp.xz);
      if(rr>DIN*0.98&&rr<DOUT){
        float g=diskShine(hp,rr);
        float temp=pow(DIN/rr,0.75);
        vec3 tint=bb(temp);
        vec3 tang=normalize(vec3(-hp.z,0.0,hp.x));
        float beta=min(sqrt(0.5/rr),0.65);
        float dop=1.0/(1.0+beta*dot(tang,v));
        float beam=clamp(dop*dop*dop,0.2,3.4);
        float gred=sqrt(max(1.0-1.0/rr,0.0));
        vec3 e=tint*g*beam*gred;
        e+=vec3(1.3,1.05,0.75)*exp(-(rr-DIN)*1.4)*0.35*beam;
        acc+=e*1.05*trans;
        trans*=0.35;
        if(trans<0.02)break;
      }
    }
    float rrp=length(p.xz);
    if(abs(p.y)<1.2&&rrp>DIN*1.15&&rrp<DOUT*0.8){
      float hz=exp(-abs(p.y)*2.6)*exp(-(rrp-DIN)*0.55);
      acc+=vec3(1.2,0.85,0.55)*hz*0.004*dt*trans;
    }
    p=np;
  }

  vec3 col=acc;
  if(captured<0.5){
    vec3 dome=normalize(v);
    vec3 neb=vec3(0.30,0.22,0.50)*fbm3(dome*3.0+3.0)+vec3(0.12,0.18,0.40)*fbm3(dome*1.45+8.0);
    float band=exp(-abs(dome.y+0.18)*2.4);
    neb+=vec3(0.38,0.30,0.55)*band*(0.35+0.65*fbm3(dome*4.1+17.0));
    neb+=vec3(0.10,0.16,0.34)*band*fbm3(dome*8.4+31.0)*0.8;
    float glint=starfield(dome,72.0,0.9935,0.0)+starfield(dome,168.0,0.996,41.7)*0.7+starfield(dome,30.0,0.991,11.3)*1.3;
    col+=(neb*0.62+vec3(0.9,0.93,1.0)*glint*1.6)*trans;
  }

  col=1.0-exp(-col*0.85);
  float vin=0.80+0.20*(1.0-smoothstep(0.95,2.20,aim));
  col*=vin;
  float lum=dot(col,vec3(0.299,0.587,0.114));
  float alpha=clamp(lum*2.3+captured*0.85*(1.0-smoothstep(1.05,2.30,aim)),0.0,1.0);
  gl_FragColor=vec4(col*alpha,alpha);
}
`;

const downSrc = `precision highp float;
uniform sampler2D tex;
uniform vec2 res;
uniform float cut;
varying vec2 vUv;
void main(){
  vec2 px=1.0/res;
  vec4 s=texture2D(tex,vUv)*4.0;
  s+=texture2D(tex,vUv+vec2(px.x,px.y));
  s+=texture2D(tex,vUv+vec2(-px.x,px.y));
  s+=texture2D(tex,vUv+vec2(px.x,-px.y));
  s+=texture2D(tex,vUv+vec2(-px.x,-px.y));
  s/=8.0;
  s.rgb=max(s.rgb-vec3(cut),vec3(0.0));
  gl_FragColor=s;
}`;

const upSrc = `precision highp float;
uniform sampler2D tex;
uniform sampler2D add;
uniform vec2 res;
varying vec2 vUv;
void main(){
  vec2 px=1.0/res;
  vec4 s=texture2D(tex,vUv)*4.0;
  s+=texture2D(tex,vUv+vec2(px.x,0.0))*2.0;
  s+=texture2D(tex,vUv-vec2(px.x,0.0))*2.0;
  s+=texture2D(tex,vUv+vec2(0.0,px.y))*2.0;
  s+=texture2D(tex,vUv-vec2(0.0,px.y))*2.0;
  s+=texture2D(tex,vUv+px);
  s+=texture2D(tex,vUv-px);
  s+=texture2D(tex,vUv+vec2(px.x,-px.y));
  s+=texture2D(tex,vUv+vec2(-px.x,px.y));
  gl_FragColor=s/16.0+texture2D(add,vUv);
}`;

function isFancy() { return props.detail === 'detailed' || props.detail === 'new'; }
function getSceneSrc() {
  if (props.detail === 'new') return sceneNewSrc;
  return props.detail === 'detailed' ? sceneDetSrc : sceneSrc;
}
function drawScale()   { return isFancy() ? 1.0 : 0.72; }
function dprCap()      { return isFancy() ? 2.0 : 1.25; }

function compile(kind: number, src: string): WebGLShader | null {
  if (!gl) return null;
  const s = gl.createShader(kind);
  if (!s) return null;
  gl.shaderSource(s, src); gl.compileShader(s);
  if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) { console.error(gl.getShaderInfoLog(s)); gl.deleteShader(s); return null; }
  return s;
}

function link(frag: string): WebGLProgram | null {
  if (!gl) return null;
  const vs = compile(gl.VERTEX_SHADER, vertSrc);
  const fs = compile(gl.FRAGMENT_SHADER, frag);
  if (!vs || !fs) return null;
  const p = gl.createProgram();
  if (!p) return null;
  gl.attachShader(p, vs); gl.attachShader(p, fs); gl.linkProgram(p);
  if (!gl.getProgramParameter(p, gl.LINK_STATUS)) { console.error(gl.getProgramInfoLog(p)); return null; }
  return p;
}

function bindMesh(prog: WebGLProgram) {
  if (!gl) return;
  const slot = gl.getAttribLocation(prog, 'spot');
  gl.enableVertexAttribArray(slot);
  gl.vertexAttribPointer(slot, 2, gl.FLOAT, false, 0, 0);
}

function wire(): boolean {
  if (!gl) return false;
  film = null; downs = []; ups = [];
  sceneProg = link(getSceneSrc());
  postProg  = link(postSrc);
  downProg  = link(downSrc);
  upProg    = link(upSrc);
  if (!sceneProg || !postProg || !downProg || !upProg) return false;
  meshBuf = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, meshBuf);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1,-1,3,-1,-1,3]), gl.STATIC_DRAW);
  bindMesh(sceneProg); bindMesh(postProg); bindMesh(downProg); bindMesh(upProg);
  const hf  = gl.getExtension('OES_texture_half_float');
  const hfl = gl.getExtension('OES_texture_half_float_linear');
  texType = hf && hfl ? hf.HALF_FLOAT_OES : gl.UNSIGNED_BYTE;
  return true;
}

function rebuild() {
  if (!gl) return;
  const sp = link(getSceneSrc());
  const pp = link(postSrc);
  if (!sp || !pp) return;
  sceneProg = sp; postProg = pp;
  gl.bindBuffer(gl.ARRAY_BUFFER, meshBuf);
  bindMesh(sp); bindMesh(pp);
  if (pane.value) pane.value.width = 0;
  born = 0;
}

function makeTarget(w: number, h: number): Target | null {
  if (!gl) return null;
  const tex = gl.createTexture();
  const fb = gl.createFramebuffer();
  if (!tex || !fb) return null;
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, w, h, 0, gl.RGBA, texType, null);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
  gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
  const ok = gl.checkFramebufferStatus(gl.FRAMEBUFFER) === gl.FRAMEBUFFER_COMPLETE;
  gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  if (!ok) { gl.deleteTexture(tex); gl.deleteFramebuffer(fb); return null; }
  return { tex, fbo: fb, w, h };
}

function dropTargets() {
  if (!gl) return;
  for (const t of [film, ...downs, ...ups]) {
    if (t) { gl.deleteTexture(t.tex); gl.deleteFramebuffer(t.fbo); }
  }
  film = null; downs = []; ups = [];
}

function buildTargets(w: number, h: number) {
  if (!gl) return;
  dropTargets();
  film = makeTarget(w, h);
  if (!film && texType !== gl.UNSIGNED_BYTE) {
    texType = gl.UNSIGNED_BYTE;
    film = makeTarget(w, h);
  }
  if (!film) return;
  let dw = w, dh = h;
  for (let i = 0; i < 4; i++) {
    dw = Math.max(1, dw >> 1);
    dh = Math.max(1, dh >> 1);
    const t = makeTarget(dw, dh);
    if (!t) { dropTargets(); return; }
    downs.push(t);
  }
  for (let i = 2; i >= 0; i--) {
    const t = makeTarget(downs[i].w, downs[i].h);
    if (!t) { dropTargets(); return; }
    ups.push(t);
  }
}

function resize() {
  const cv = pane.value;
  if (!cv || !gl) return;
  const dpr = Math.min(window.devicePixelRatio || 1, dprCap());
  const w = Math.max(8, Math.round(cv.clientWidth  * dpr * drawScale()));
  const h = Math.max(8, Math.round(cv.clientHeight * dpr * drawScale()));
  if (cv.width === w && cv.height === h && film) return;
  cv.width = w; cv.height = h;
  buildTargets(w, h);
}

function paint(t: number) {
  const cv = pane.value;
  if (!cv || !gl || !sceneProg || !postProg || !downProg || !upProg) return;
  resize();
  const filmT = film;
  if (!filmT || downs.length < 4 || ups.length < 3) return;
  driftX += (aimX - driftX) * 0.03;
  driftY += (aimY - driftY) * 0.03;

  gl.bindFramebuffer(gl.FRAMEBUFFER, filmT.fbo);
  gl.viewport(0, 0, filmT.w, filmT.h);
  gl.useProgram(sceneProg);
  gl.uniform2f(gl.getUniformLocation(sceneProg, 'res'),   filmT.w, filmT.h);
  gl.uniform1f(gl.getUniformLocation(sceneProg, 'time'),  t);
  gl.uniform2f(gl.getUniformLocation(sceneProg, 'drift'), driftX, driftY);
  gl.drawArrays(gl.TRIANGLES, 0, 3);

  gl.useProgram(downProg);
  let src: Target = filmT;
  for (let i = 0; i < downs.length; i++) {
    const dst = downs[i];
    gl.bindFramebuffer(gl.FRAMEBUFFER, dst.fbo);
    gl.viewport(0, 0, dst.w, dst.h);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, src.tex);
    gl.uniform1i(gl.getUniformLocation(downProg, 'tex'), 0);
    gl.uniform2f(gl.getUniformLocation(downProg, 'res'), src.w, src.h);
    gl.uniform1f(gl.getUniformLocation(downProg, 'cut'), i === 0 ? 0.30 : 0.0);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    src = dst;
  }

  gl.useProgram(upProg);
  let carry: Target = downs[3];
  for (let i = 0; i < ups.length; i++) {
    const dst = ups[i];
    const add = downs[2 - i];
    gl.bindFramebuffer(gl.FRAMEBUFFER, dst.fbo);
    gl.viewport(0, 0, dst.w, dst.h);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, carry.tex);
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, add.tex);
    gl.uniform1i(gl.getUniformLocation(upProg, 'tex'), 0);
    gl.uniform1i(gl.getUniformLocation(upProg, 'add'), 1);
    gl.uniform2f(gl.getUniformLocation(upProg, 'res'), carry.w, carry.h);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    carry = dst;
  }

  gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  gl.viewport(0, 0, cv.width, cv.height);
  gl.useProgram(postProg);
  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, filmT.tex);
  gl.activeTexture(gl.TEXTURE1);
  gl.bindTexture(gl.TEXTURE_2D, carry.tex);
  gl.uniform1i(gl.getUniformLocation(postProg, 'tex'),   0);
  gl.uniform1i(gl.getUniformLocation(postProg, 'bloom'), 1);
  gl.uniform2f(gl.getUniformLocation(postProg, 'res'),   cv.width, cv.height);
  gl.uniform1f(gl.getUniformLocation(postProg, 'time'),  t);
  gl.uniform1f(gl.getUniformLocation(postProg, 'detail'), isFancy() ? 1.0 : 0.0);
  gl.drawArrays(gl.TRIANGLES, 0, 3);
}

function loop(stamp: number) {
  if (dozing) return;
  const cv = pane.value;
  if (cv && cv.clientWidth > 0) { if (!born) born = stamp; paint((stamp - born) / 1000); }
  frameId = requestAnimationFrame(loop);
}

function onPointer(e: PointerEvent) {
  aimX = Math.max(-1, Math.min(1, (e.clientX / window.innerWidth  - 0.5) * 2));
  aimY = Math.max(-1, Math.min(1, (e.clientY / window.innerHeight - 0.5) * 2));
}

function nap() {
  const sleep = document.hidden || !props.active;
  if (sleep && !dozing)      { dozing = true;  cancelAnimationFrame(frameId); }
  else if (!sleep && dozing) { dozing = false; born = 0; frameId = requestAnimationFrame(loop); }
}

watch(() => props.active, nap);
watch(() => props.detail, rebuild);

function onCtxLost(e: Event) { e.preventDefault(); cancelAnimationFrame(frameId); }
function onCtxRestore()      { if (wire()) { born = 0; frameId = requestAnimationFrame(loop); } }

function boot() {
  const cv = pane.value;
  if (!cv) return;
  gl = cv.getContext('webgl', { alpha: true, premultipliedAlpha: true, antialias: false, depth: false, stencil: false });
  if (!gl || !wire()) return;
  paint(0);
  ready.value = true;
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) { paint(12); return; }
  cv.addEventListener('webglcontextlost',     onCtxLost);
  cv.addEventListener('webglcontextrestored', onCtxRestore);
  document.addEventListener('visibilitychange', nap);
  window.addEventListener('pointermove', onPointer);
  frameId = requestAnimationFrame(loop);
}

onMounted(boot);
onUnmounted(() => {
  cancelAnimationFrame(frameId);
  document.removeEventListener('visibilitychange', nap);
  window.removeEventListener('pointermove', onPointer);
  const cv = pane.value;
  if (cv) { cv.removeEventListener('webglcontextlost', onCtxLost); cv.removeEventListener('webglcontextrestored', onCtxRestore); }
  if (gl) { const ext = gl.getExtension('WEBGL_lose_context'); if (ext) ext.loseContext(); }
});
</script>

<style scoped>
.hole-layer {
  position: fixed; inset: 0; width: 100%; height: 100%;
  pointer-events: none; z-index: 0; background: transparent;
  opacity: 0; transition: opacity 800ms ease;
}
.hole-layer--live { opacity: 1; }
</style>
