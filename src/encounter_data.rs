use sdl3::rect::Rect;

use crate::{compute_dust_search::DustSearchMode, dust::{DustData, DustSearchConfig}, rng::PrecomputedRNG};

const DUST_FROGGIT: DustData = DustData::new_kill(
    "yi}yi}yi}a/d.e}`1a1d}_3_3c}^5]5b}^,Z-\\6a}],\\-Z.Y.`}]+^,Z-[-`}]+^,Z,],`}]+^-X-],`}]+\\/X-],`}]+Z.W,W/Y,`}]6W,W1W,`}]L)`}[L+`}ZL,`}YL._}X5^*]4^}W3m3]}W1[.Z.Y3\\}W0Y>Y1\\}WL3\\}XL1]}YL/^}ZL,`}\\Kc}`Bh}yi}yi}yi}b,c+k}`Ai}_4V6h}_3X6g}_0V*Y+V1g}_/X)Y*X0g}^0W0X1f}^Ff}^Ff}^1a1f}^/d0f}^.X+Z+X/f}^.W*V)Z)W)X.f}^-X)W)Z)W)Y.e}^-X)V)[)V*Y.e}^-X+\\+Y/d}].X*^*Y0c}Z1\\,_1a}Y2[)Y)^2`}Y1\\)Y)_1`}Z/o.a}yi}yi}yi}yi}~~~",
    false,
    DustSearchMode::SecondToLastFrame
);

const DUST_WHIMSUN: DustData = DustData::new_kill(
    "ye}ye}ye}f*g)c}e)W)e)V)b}d)Y)d)V)b}c+X)c)X)a}c+X)c)X)a}j)Z.W)X*`}j)X1V)X+_}j)V4Y*`}j+X*X+f}j)Z*Y+e}i)Z,Y*e}h*Y-Z*d}g*Y.Z*d}g*X0Y*d}f<d}f+V-V.V,c}f0X2c}f0X2c}f/Z1c}b)X.\\0c}a*X.\\0c}`+W/V)W)W0c}_*V)W>W*_}]*X)W>W*_}[*V)W)W?V+_}Z+W+W-W.W/W*V)^}Y*V)W*W-Y,Y.W,^}W,V,X,Z,Y.W+V)]}X-W)X*W+W,V)W.X*V*\\}`+W6W-X-\\}c:W+X*X)[}b=X+W*V)[}b@[,Z}b?e}b)X*X+X0e}g)Y)Y*X*f}ye}ye}k)[)k}k)[)k}k)[)k}k)[)k}i+[+i}h+\\,h}g+^,g}g*a*g}ye}ye}ye}ye}~~~",
    false,
    DustSearchMode::SecondToLastFrame
);

const DUST_MOLDSMAL: DustData = DustData::new_kill(
    "yd}yd}yd}d.X+W,g}b*[)V)W)V)Y*e}a/W)V)V)V)W/c}h+W)W+j}_1\\)X1a}^)],\\+\\*_}])W/c.X)^}\\,p*W)]}[*X1_0W)W)\\}[)V+Y*c*Z*V)W)[}Z)V)Y*Y+W*Y)X+Y)V)V)[}Y)V)Y)W.W+V*W+X)Y)V)V)Z}Y)V)X)V*Y*V,V+W-V)Y)V)V)Y}X)V)X)V*Z)V*W)V)V*W-V)Y)V*Y}X)V)W)V*Z)V*X)V)W*V)W*V*V)V)V)V)X}W)V)X)V)[)V)Y)V)X)V)X*V)Y)V)Y}W)V)X)V)V*X)V)Y)V)X)V)Y*V)X)V)Y}W)V)W)V)W*X)V)Y)V)X)V)Y*V)V)W)V)X}X)X)V)V+X)V)V)W)V)X)V)V*W)V)V)W)V)X}X)W*V)V)Y)V)W)W)V)V)V)V)W)W)V)Z)Y}W)X)V*[)V)V*W)V)V)V)V)W*V*V)Y)Y}W)X)V)[)V)W*W)V)V)W)V)W)W)V)Y*X}W)X)V)W)X)V)W)X)V)V)W)V*V)W)V)Z)X}W)X)V)V)Y)V)[)V)V)X)V)Y*V)Y)X}X)V*V)V)Y)V)[)V)Z)V)Z)V)Y)X}X)V)V*V)X)W)V)X)V)[)V)W)W)V)X*X}X)V)V)[)V*V)X)V)[)V)W)X)V)W)Y}Y*V)[)V)W)X)V)W)X)V*Z)V+Z}\\)[)V)W)X)V)W)Y)V)[)]}\\)Z)V*[)V)W)Y)V)[)]}\\)Z)V)\\)V)W)Y)V)[)]}])Y)V)\\)V)\\)V)Y*^}^,W)\\)V)\\)W-_}d)\\)V)\\)f}e)[)V)[)g}f.X.h}yd}yd}yd}yd}~~~",
    false,
    DustSearchMode::LastFrame
);

const DUST_MIGOSP: DustData = DustData::new_kill(
    "ye}ye}b+f+e}a-d-d}_/c*V,c}^-W*b)X,b}]-Y)a*Y,a}]+[*`)[+a}g)`)\\*a}g*^*j}h)V/V)k}g)W/m}f*X-X)j}e,W-W+i}d:h}c<g}c<g}b*[.[*f}b*\\,\\*f}b*]*]*f}a+g+e}a,\\*\\,e}]+V@e}\\+V/W)V)V)V)W/V+a}[+W-d-W+`}[*X,V)b)V-W+_}[*X,V+V/V,V,X*_}[*X-X1Y-X*_}[*XBX*_}[-]1_,_}[,_/_-_}`-Z-Z.d}`-W)W-W)W.d}`.W)V-V)W/d}`Bd}a@e}a+f,e}a@e}b)i)f}b*g*f}c<g}d:h}ye}ye}ye}g*\\*l}g*\\*l}g*\\*l}c.\\.h}a0\\0f}`0^0e}ye}ye}~~~",
    false,
    DustSearchMode::LastFrame
);

const DUST_LOOX: DustData = DustData::new_kill(
    "ye}ye}w+d}a*j,b}`*l,a}_+l-`}^+n,`}],n-_}\\-n-_}\\-^1\\-_}\\-[7Y-_}\\-Y;W-_}\\.WC_}\\K_}\\6_3_}]2d0`}^/h.`}^.j-`}].l,`}]-m-_}]-n,_}\\-o-^}\\,`._-^}\\,^1_-]}[-]+[*^-]}[-]*])^-]}[-\\*_)].\\}[-\\*Y*Y)].\\}[-\\*X)W)X)].\\}Z.\\*X)W)X)].\\}Z.\\*X)W)X)]/[}Y/\\*Y*Y)\\0[}Y0\\*])]1Z}X1\\+[*]1Z}X2\\1]3Y}W3^.^+X-Y}W-Y+k*[-X}V-\\)k)],X}V,^)i)_,W}V+`)g)a+W}V+a*c*b+W}V+c6d+W}V+e2f+W}V+y\\+W}V+y\\,V}V.yX-V}V/yV.V}W.a)\\)e*V+V}X+V)a)V)X)W*c)W+V}X)V)a)V)V*V+V*Z)])W*W}X)V)e+V*])_*X}X)`)i)d}d)i)d}d)h*d}d)g*e}d*e+e}c,d.b}a.c0a}`0b0a}ye}ye}~~~",
    false,
    DustSearchMode::LastFrame
);

const DUST_VEGETOID: DustData = DustData::new_kill(
    "ye}m)p}n)o}i*X)W+j}e+X)W)V)Y+f}h)W*V)V)W*i}c-V)W)V)V)V)W-d}b)Z)V)V)V)V)W*Z)c}c,W)W)V)V)V)X,d}a*Y)W)V)V)V)X*Y*b}`)\\)V)V)V)V*V)])a}i)W)V*V*^)a}j)V*V*V*j}j)W)V*V)k}m)V*m}c+e+e}`1`0b}_G`}^I_}]J_}]?X1^}\\?Z0^}\\1Y1\\/^}\\0[0\\/^}\\0\\/\\/^}\\0\\/X3^}\\0\\/W4^}]3X/W4^}]4W/W3_}]4W<_}^3W<_}^H`}^H`}^6W2W,`}_,X.W*W.W,`}_-X,W+W-W,a}`,X2W+W-a}`-X6W,b}a,Y4W-b}a-X3X,c}b-X2W-c}c-W1W-d}c-X/X-d}d-Y*Z-e}d._,f}e-^-f}f-\\-g}f.X)V.g}g7h}h5i}i3j}j1k}k.m}ye}ye}~~~",
    false,
    DustSearchMode::LastFrame
);

const DUST_SNOWDRAKE_CHILLDRAKE: DustData = DustData::new_kill(
    "yyc}yyc}yyc}yyc}yyc}yyc}j+y+l}j,\\+[*[+\\,l}j-Z-Z*Z-Z-l}j.X.Y,Y.X.l}k.W-Z,Z-W.m}k/V,V*X,X*V,V/m}k4V+W,W+V4m}l4V0V+V4n}l4W2W4n}m3X0X3o}h7Z.Z7j}g2W,[,[,W2i}g2Y*[,[*Y2i}g3Y)[,[)Y3i}h,V-`,`-V,j}i*V)V-_,_-V)V*k}j)V*V-^,^-V*V)l}l+V*W)],])W*V+n}f,X*V+_,_+V*X,h}g,W*V,^,^,V*W,i}h0W+],]+W0j}j0W)],])W0l}j2].]2l}i6Z.Z6k}h,X2W0W2X,j}c-^D^-e}a8Y<Y8c}`=Y4Y=b}`+V<W2W<V+b}`*X<W0W<X*b}`+V=W0W=V+b}`AW0WAb}`-X:W.W:X-b}a+Z9W.W3Y*Z+c}a+Z-Y0W.W0[+Z+c}a,X-W*X.W.W.X*W-X,c}b2W-X,W.W,X-W2d}b2W/W+W.W+W*W+W2d}c.Z+W+V+W.W+c.e}d.b+W.W+b.f}e0_+V0V,^0g}f/W.W*V,W,V2W/h}g.V0V)V,V*V,V2V.i}h7V,V,V,V7j}i5V,V.V,V5k}j3V,V0V,V3l}k)W.V,V2V,V.p}l)Y*V,V4V,V*Y)n}m,W,V,W*W,V,W,V)m}g.V0V,W,W,V0V-j}e1X,V)V+W,W+V)V,X0h}d4V+V)X*W,W*X)V+V3g}d3V,Y)V2V)Y,V2g}e1V*V*Y*V0V*Y*V*V0h}i,V*W*Z)W.W)Z*W*V,k}l)X)V*^,^*V)X)n}k.V*])V*V)]*V.m}i0V*]*W*]*V0k}h1V*]*W)^*V1j}g3V)^)])X)V3i}f4X)d*V)X4h}f4X*W)`)V*X4h}e0V-X*V*`*X-V0g}e.X.X*V)_*X.X.g}d.X0X*_*X0X.f}d+Z0W*V*V*Z*V*W0Z+f}k0W+W*V)V*V*W+W0m}j0W,X*W)V*X,W0l}j/Y*V)X*W*X)V*Y/l}i-Y*X*Y,Y*X*Y-k}i+Y1V)X*X)V1Y+k}p1V+Y+V1r}p1V2V1r}p1W0W1r}p1V)])V1r}p1V2V1r}p1V2V1r}q0V2V0s}q0W0W0s}r/V)])V/t}s.V2V.u}t-V2V-v}s)V,V2V,V)u}r+Z2Z+t}q+k+s}p+m+r}o+o+q}k*V+q+V*m}l,s,n}m+s+o}l-q-n}k+W*o*W+m}j+Y*m*Y+l}yyc}yyc}~~~",
    true,
    DustSearchMode::LastFrame
);

const DUST_ICE_CAP: DustData = DustData::new_kill(
    "yf}o)o}o)o}o)o}n+n}n+n}n+n}m-m}m-m}m-m}l)V+V)l}l*V)V*l}k)V*V*V)k}k*V+V*k}k+V)W*k}k,X*k}j-Y*V)h}g)Y+Z)j}i)W+p}l+W)X)i}l+p}j)W)Z)k}i*W)Z*j}i*^*i}i+]*i}i+W)Z*i}h/Z+h}f)V/[*h}h/[*h}f)V.\\*h}g/\\+g}e)V/\\+g}g/\\+g}e)V/]*g}d)V0]+d)V}V)b)V0]+b*W}W)a)V0]+a*V)V}V)V)`)V0^*`*V)W}V)W)^)V1^+^+V)W}W)V*])V0V)]+[)W*V)X}W)W*\\)V0V)]+\\)V)W)X}W*W*[)V0V)]+Y)V*X)Y}X)X*Y)V1V*]+Y+W*Y}X)X+X)V1V+\\+V)V+W*Z}X*X+W)V1V)V)\\,V+X*Z}Y)Y,V2V)V*[+V+X*[}Y)Z*W2V)W)[*V+Y*[}Y*Z)V3V)W*\\+W)V*\\}Z)Z)V3V)X)[+W)W*\\}Z*Z4V)X*Z*Z*]}Z*Y5V)Y)Z)W)W+]}X)V+X*W0V*Y*X)W)X*^}X*W)W)W*V/V)Y+X*Y+^}X,W)V-V.V)X+^+_}X)W*W/V-V)V,W*V+V,_}X)V)W2V,V,W*V)V,V*`}Y)V)V3V+V*W*X*V,V)`}Y)W5V*V)V*[)V-V)_}Z)V6Y2g}[8b+d}[Hd}[Hd}[Ic}[6[5c}\\4V.W3c}\\3V1V1d}\\2V2W/V*V*V-Y}]1V-V,W.V*V+V,Z}]1V2W-V+V+V)]}^0W0X-V+V+_}^1W.X.V,b}_1^0f}`1[3e}a:Y+e}b7W,h}d4V/g}f8i}i1m}yf}yf}k,p}h)V.V)m}h*V,V*m}yf}j*W*o}j*W*o}yf}yf}~~~",
    false,
    DustSearchMode::LastFrame
);

const DUST_DOGGO: DustData = DustData::new_kill(
    "yt}yt}yt}yt}yt}yt}yt}yt}l)V)d)V)l}l*V)b)V*l}l+V)`)V+l}l)W)V)^)V)W)l}l)W)V)^)V)W)l}l)X)V)\\)V)X)l}l)X)V)\\)V)X)l}l)X)V1V)X)l}l)h)l}l)i)k}l)i)k}k)j)k}k)W.X.Y)k}j*V*X*X*X*X)k}f,X/X/X)k}a1Y*`*X*k}`)X.])Z)[+b+[}`)X/])X)Z-b+[}`)W9X3b+[}aIb+[}a<Z)V.k}b4\\*V*V/^/[}c1X*V*V*V)W/]1Z}r*V)Z/\\2Z}l-[,V.\\2Z}m=\\2Z}n:^1[}q5`+a}y^)[+V0X}y]*[+a}p)d+[-V)V)[}p*c,Z-V)V)[}o+X0X-Y-V)V)[}n,V4V-Y-V)V)[}j)W-V4V,W)W-V)V)[}k)W,V4V,V)X-V)V)[}c2V,V3W,V1V)V)[}b3V,W0Y,V1V)V)[}b3V,Y-Y-V1V)V)[}b3V-W/X-V1V)V)[}b3V-X.W.V0W)V)[}b.W+V.W-W/V+\\)V)[}b-Y*V/W+W0W*\\)V)[}b-X*W1X2X)\\)V)[}b-X)Y=`)V)[}b-].V-V0`)V)[}b-]-V)V+V)V.a)V)[}b-^,V*X*V.a)V)[}b-^,V/V.a)V)[}b-^+V*V+V*V-a)V)[}b-^*W+X+W,a*\\}b-W)[*V*V*V*V*V,a*\\}b-W)[*W*Z*W,a*\\}a-X)[,V)V+V)V.a)]}^)V.V)V.V,V)Z)V.V/Y)]}^)V.V)V.V,V*X*V.V0X)]}^)V.V)V)[-V-V/])a}^)V.V)V.V.Z0V/b}`-Y)[;j}a+V*W)^4n}p+a.h}p-W)W)W.j}o/V)W)V0i}t*[,m}p,a+j}p<i}p<i}o.W2W*h}o9X*h}o*W/X+X+g}n*X/X1g}n*W<g}mAg}m>W)g}m>W)g}m-X,W0W)g}m-W-W+W.g}m3Y*W.g}m2Z2g}n*W,\\1g}n*W+^,W)h}n.X-X+W)h}o-W/X-h}o-W/X,i}o-W0W,i}o,Y/W+j}o,Y/W+j}x.o}o)V)[.W)V)j}o)V)\\-W)V)j}o)V)],W)V)W*X)b}o)V)^+W)V)W.b}yY)^,b}n-a.Y)c}l*Z)`)Z*f}k)])_)\\)e}k)W)W)W)_)Y)V)V)d}k)W)W)W)_)Y)V)V)d}k2_2d}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}~~~",
    true,
    DustSearchMode::LastFrame
);

const DUST_LESSER_DOG: DustData = DustData::new_kill(
    "ye}ye}ye}ye}ye}ye}ye}ye}ye}y`)Y}y`)Y}y_+X}y_+X}y_+X}y_+X}y^*V*W}y^*V*W}y^*V*W}y^*V*W}y^*V*W}y^*V*W}y^*V*W}y^*V*W}y^*V*W}y^*V*W}y^*V*W}y^*V*W}y^*V*W}y^*V*W}m*^*]*V*W}l,\\,\\*V*W}m,V,V+]*V*W}m,V1\\*V*W}l7\\*V*W}c-Y8[*V*W}_2X2X+[*V*W}]-Y+W-X+X,Z*V*W}Z-]+V-X+X)W)Z*V*W}X,])W+V+V)X3Y*V*W}W*`*W*V6V-X*V*W}V*_,W*W1X*V+Y*V*W}V)`,W*V2X*V,X*V*W}V)W)^,V*V.W+V*V,Y*V*W}V)V+],V*X-\\.X*V*W}V-],V*W-V)[+[*V*W}V,_*W*V)V-V)V*V-Z*V*W}V,_)X*V*V-Y,W)Y*V*W}V,_)X*V*V)V4V)Y*V*W}V)V*_)X*V*W2X+X*V*W}V)V*_*X*V*V)V)V,V*V-W*V*W}V)W)_,V*V+V)W,X.W*V*W}V)W)_,V*V.V*V)V0W*V*W}V)V*_,V*V/V)W1W*V*W}V)V*_,V*V0W,V)V,V*V*W}V)V*_*X*V1V)X*V,V*V*W}W)V)_*X*V.Z.V+W+X}W)V*]*Y*V)Z,V.V*X+X}W)V*]*Y*V2V.V)W*X*V}W)W*\\*Y*V2V.X)\\)}W)W+[+X*V2V/X/V}W)W+W)X+X*V2V/Z)V)X}W)X*W*W+X*V2V/W*V)Z}W)X*W*Y)X*V2V.X*W*X}W)W*X*]*V2V.X*V,W}X)V*X*]*V2V-V)W*V,W}X)\\)]*V1W,W)X)W*X}X)e*V)V.V)[)[)Z}X)e*V*c)[)V)X}Y)e*d*[)V)X}Y)`)Y*b+\\)V)X}Y)`)Y*]/V)[)V)X}Z)_)Y*V3X)V)Z+X}Z)W)Y)W*X*a+V)V)_}Z)W*V+W*X*V5V*V)_}[)V*W*W*X*V2X*V*_}[)W)W*\\*`,V,^}[)W)W*\\*V3X-^}[)W)W*\\*a1]}\\)Z)\\*V3V0]}\\)Z)\\*V4V/]}\\)Z)\\*V5V.]}])Y)\\*V5X-\\}])a*V4[+\\}])a*V.[-W*\\}^)`*\\)X0W)\\}^)`*V.Y0_}_)^*W.Z/_}_)^*V/Z/_}_)^*V.[.`}`)]*V.\\-`}`)]*V-],a}a)[+V,_+a}a)[*X+_+a}b)Z*X+`*a}b)Z*X*a*a}c)Y*X*a*a}c*X*X*a*a}d*V+W+a+`}d.W*c*`}e-t}f+u}ye}~~~",
    false,
    DustSearchMode::LastFrame
);

const DUST_JERRY: DustData = DustData::new_spare(208.0, 104.0);

const DUST_AARON: DustData = DustData::new_kill(
    "yt}yt}yt}yt}yt}yt}yt}yt}l)V)d)V)l}l*V)b)V*l}l+V)`)V+l}l)W)V)^)V)W)l}l)W)V)^)V)W)l}l)X)V)\\\\)V)X)l}l)X)V)\\\\)V)X)l}l)X)V1V)X)l}l)h)l}l)i)k}l)i)k}k)j)k}k)W.X.Y)k}j*V*X*X*X*X)k}f,X/X/X)k}a1Y*`*X*k}`)X.])Z)[+b+[}`)X/])X)Z-b+[}`)W9X3b+[}aIb+[}a<Z)V.k}b4\\\\*V*V/^/[}c1X*V*V*V)W/]1Z}r*V)Z/\\\\2Z}l-[,V.\\\\2Z}m=\\\\2Z}n:^1[}q5`+a}y^)[+V0X}y]*[+a}p)d+[-V)V)[}p*c,Z-V)V)[}o+X0X-Y-V)V)[}n,V4V-Y-V)V)[}j)W-V4V,W)W-V)V)[}k)W,V4V,V)X-V)V)[}c2V,V3W,V1V)V)[}b3V,W0Y,V1V)V)[}b3V,Y-Y-V1V)V)[}b3V-W/X-V1V)V)[}b3V-X.W.V0W)V)[}b.W+V.W-W/V+\\\\)V)[}b-Y*V/W+W0W*\\\\)V)[}b-X*W1X2X)\\\\)V)[}b-X)Y=`)V)[}b-].V-V0`)V)[}b-]-V)V+V)V.a)V)[}b-^,V*X*V.a)V)[}b-^,V/V.a)V)[}b-^+V*V+V*V-a)V)[}b-^*W+X+W,a*\\\\}b-W)[*V*V*V*V*V,a*\\\\}b-W)[*W*Z*W,a*\\\\}a-X)[,V)V+V)V.a)]}^)V.V)V.V,V)Z)V.V/Y)]}^)V.V)V.V,V*X*V.V0X)]}^)V.V)V)[-V-V/])a}^)V.V)V.V.Z0V/b}`-Y)[;j}a+V*W)^4n}p+a.h}p-W)W)W.j}o/V)W)V0i}t*[,m}p,a+j}p<i}p<i}o.W2W*h}o9X*h}o*W/X+X+g}n*X/X1g}n*W<g}mAg}m>W)g}m>W)g}m-X,W0W)g}m-W-W+W.g}m3Y*W.g}m2Z2g}n*W,\\\\1g}n*W+^,W)h}n.X-X+W)h}o-W/X-h}o-W/X,i}o-W0W,i}o,Y/W+j}o,Y/W+j}x.o}o)V)[.W)V)j}o)V)\\\\-W)V)j}o)V)],W)V)W*X)b}o)V)^+W)V)W.b}yY)^,b}n-a.Y)c}l*Z)`)Z*f}k)])_)\\\\)e}k)W)W)W)_)Y)V)V)d}k)W)W)W)_)Y)V)V)d}k2_2d}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}yt}~~~yj}h2\\)V)g}f*_/V)V)f}d*]*_)W)e}]/^+c)d}])c,c)d}^*a,c)d}`)_,c)e}`)_,V+^)f}a)]7V*g}],]6V)j}\\)a3V*V*i}]+^0Y*l}])_0V)V)V*V)V)V.a}\\*_0Z+X+V*V)`}\\)`>V-_}[)aD_}[)aD_}[)a5Y3_}[)b4V+V1`}Z)W*_9i}Y)W)W)^1W5b}Y+Y.X3o}_0W)V:f}^2W)V;d}]4V+V;b}]L)a}\\L+`}\\3V:X)X*`}[4W@V)`}[2WCa}[1VG_}[1V0V?^}[0V2V?]}Z,X*V1X>]}Z/X0V0V8]}Z0V0V1W7]}Z0V0V3V6]}[0Y+V5V5]}[4V*V5W3^}\\1W)W5W)V2^}],V-Y)V1V,X._}^+V-W)W)V/W/V,`}b,X,V+[5_}d)X-V*V;_}h-X<_}b)[C_}b,YA`}b-Z?`}b-V)Y=a}b-V*Z:b}a*V*W,[5d}a+X.\\1f}a-W/f0[}a,W)W/a+]*Y}`*V*V2\\)W+b)X}`+X2[)V*Y0Z)W}_-W1\\*Y4Y)V}^*V*W)X-\\)Y6Y)V}^+X1\\)Y8Y)}^,W0\\)Z8Y)}\\+V*V)X,\\)Z9Y)}[-X0\\)Z9Y)}Z,V*W/\\)[9Y)}Y.V)V)X+])\\+Z0Y)}X-V*W/\\)g-Z)}W/X)W,\\)i,Z)}W1V.r+Z)V}V*V0W,r+Z)V}V)W5k,Y*[)V})X4d*Y*Y)X)[)W})Y2e*W*[)W*Z)X})Z0i)])W)Z)Y})\\+e*Y)])W)Y*Z})o*X)])\\)\\})s)])\\)]}V)j*Z)])\\)^}V)j*Y)\\*\\)_}W)]*Z*])Z+\\*`}X)\\*Z*\\/^)b}Y*h,`+c}[*d+a*f}]+_+b)h}`3b*i}e*d*k}c*e)m}b)Z.[)n}a)Z/Z)o}`)Y+W+Z)p}_)Y+W+Z)q}^)Y0Z)r}])Y/[)s}])Y,])t}])d)u}\\)c*v}\\)`+x}\\)Z.yW}\\)V,y]}])ya}yj}yj}~~~",
    false,
    DustSearchMode::LastFrame
);

const DUST_ASTIGMATISM: DustData = DustData::new_kill(
    "ya}X*yW*X}W*yY*W}V+yY+V}V+yY+V}V+e0e+V}V,a6a,V}V-^:^-V}W.Y@Y.W}WL0W}XL.X}XL.X}Y9[9Y}Z8[8Z}\\6[6\\}]5[5]}]-Y-Y-Y-]}V)Z/X-Y-X/Z)V}V*X0X-Y-X0X*V}V5Y-W-Y5V}W5X-W-X5W}X4X-W-W5X}Y4W+[+W4Y}[3c3[}[1g1[}[/W*c*W/[}[.V+e+V.[}[-V+g+V-[}\\,V+g+V,\\}\\-V*g*V-\\}\\-V*g*V-\\}]-V*e*V-]}].V)e)V.]}^.V)c)V.^}^/e/^}_0a0_}])W2[2W)]}\\+W@W+\\}\\+X>X+\\}\\+Y<Y+\\}[,[8[+\\}[,^2^,[}[,Y)g)Y,[}Z+V)Y+c+Y,[}Z+V)Y0Y0Y)V+Z}Z*W)Y/[/Y)V+Z}Z*].[.]*Z}Z)^.[.^)Z}d-]-d}d-]-d}e,],e}e,],e}e,],e}f*_*f}f*_*f}g)_)g}g)_)g}ya}~~~",
    false,
    DustSearchMode::SecondToLastFrame
);

const DUST_MADJICK: DustData = DustData::new_kill(
    "yy}n/yY}l*\\*yW}k)`)yV}k)a)y}j)c)x}j)Y,\\)w}j)X)Y)\\)v}j)X)W)W)\\)u}k)X*V)W)[)u}l*X)X)\\)t}n+Z)[)t}v)\\)s}v)\\)s}v)])g.Z}k)^)^)^)\\)[)Y}j)V)])^)])V)Z)V)[)X}j)W)[)`)[)W)Y)W)\\)W}j)X.b.X)Y)X*V)V)V)W}Y.a)r)Z)Z)V)V*W}X)[)`)r)Z)Y)Z)W}W)V)[)`)p)[)Z)Y)W}V)W)\\)`)n)])X)Y)X}V)X*V)V)V)a*j*_)X)W)Y}V)Z)V)V*c+e*b.Z}V)Y)Z)f,^+o}V)Z)Y)a)]1r}W)X)Y)b,y\\}X)X)W)c.yZ}Y.d,V-],Y)i}n,W<j}n+V)Y8k}o)Y*[1m}o+W+V*V)v}p+W*V*V)V)V)V)p}q+Y*V)V)V)r}r-yW}t3s}k.c.m}i*[*_*[*k}h)_)])_)j}g)a)[)a)i}g)X,[)Y)[,X)i}g)W)Y)[)W)[)Y)W)i}g)V)[)W*V)W)V*W)[)V)i}g)V)\\)V*W*W*V)\\)V)i}h*\\)V*W*W*V)\\*j}i*\\)Y*Y)\\*k}r)Y*Y)t}s)X)Y)u}t)W)X)v}s)X)Y)u}s)X*Y)t}r)Z)X)u}s)Y)Y)t}r)Z)Z)s}r)Z)Z)s}q)a)s}j)Z)c)Z)l}j)X*e*X)l}j,i,l}k)m)m}^*V)\\*V+h*V*]*V)_}])X)[*V)X;W*V)[*W*^}`)\\)X*j*W*[*W)^}])W)\\)Z)h)e)^}])^*[)f)[)_)^}`*Y*])c*])Z)X)^}]*X)V+[)V)e*\\+W)X*^}d)])h+]*Y)_}^)b)k*b)_}_*^*m)V)^*`}`*X)X)q*X)X*a}a+V)W)s)X-b}d*V)u)W)f}f*v)V)f}e*x*f}yf)g}yy}~~~",
    true,
    DustSearchMode::SecondToLastFrame
);

const DUST_KNIGHT_KNIGHT: DustData = DustData::new_kill(
    "yyyX}o)yya}o*yy`}o+yy_}m)V+c)s)y}m)V+`,s-u}k/V)\\.a.a.t}k2Z,c+X*c,s}i6X+c*V+W*c+s}h6Y+b,Y,b+s}k2Z+`8`+s}k-V/W,],V2V,],s}g1V0V8W0W8s}h-V)W,[5V)X.X)V5t}k*W*V)V)\\2X)X.X)W3u}k*V)W)W,f)X*W*X)y^}k)W)W)V,f*a*y]}f.V)V*W*h+_,y\\}g0W)V*\\1W,_,W1u}k,V-[)_.X*X._)V/l}k,V-Z,\\0W*W0\\,V*Y,h}k2f0V*V0d)\\+e}i6V._8_/V)^)d}i6V1]6]3V)^*b}l0X4\\4\\5V)`)a}l0V7\\2\\7V)`)`}n,X8\\0\\8V)a)_}n,X9\\.\\9W)Z0^}i)Z*Y:]*]:W)W+\\*]}h)[*Y;e;W)V)a)\\}g)Y0W;c;X*j}f)Z0Y9c:b-^}e)],]8a8b+X)_}d)^,_6a6b*Z)`}c)_,V)_5_5b*[)a}b1Z*V*`4]4b*])a}a)\\)W,g3V)W)V3n)a}`)\\)W1^+Y3W3X+j)a}_)\\)W3[*^1W1]*i)`}_)\\)V4Z)b/W/a)h)`}^)\\)W._)e-W-d)g)`}^)\\)W3Y)h*Y*g)f)`}])\\)W4Y)y^)f)`}])\\)W4X)i*X*i)f)_}])\\)W.^)h)\\)h)f)_}])\\)W3Y)g)^)g)f)_}])\\)W3Y)e*X-X*e)f)_}])\\)X,`*a*X1X*u)_}\\)^)W1]4Y3Y,W+V*Z)c)_}\\)^)X0X)i*V-V*^+V+[,`)^}[)\\-c,a)W*W-W*W*X+V+W*\\*^)Y)Y}[)[)`*\\*W+V*V+V*V*V/V*V-V+V+V+V)\\*\\)X*Y}Z)[)a+^,V*V+V*W3W-V+V+V+V*]*Z)W)V)Y}Y)[)b+[.V+V+V*W3W-W*V+V+V*_)Z*W)Y}X)Y+c+\\,V,V*V+X+X+Y,^*V+`-Y)Y}V/f+a+V+V+X*Z*Y+V)`*i)Z}n+`,V+V*Y*Z*Y*V)w)Z}n+`,V+V*h)w)[}n+_,V,V*f)V)e)f)[}n+_+W+V+g)g)d)\\}n+_)Y+V+g)g)d)\\}n+d*W+g)g)c)]}n+^)Y+V+X)`)X)g)b)^}n+^)Y*W+X)`)X)f)V)`)_}n+])Z)W,W*X)X)X*W)f)V)V+Z*`}n+])]+X+W*V*W+W)e)W)Y-b}n+\\)]+Y+V+V+V+V*d)X)k}n+\\)e5W)d)Y)k}n+[)h1Y)c)Z)k}n+[)f)`)W)a*[)k}n+[)g*\\*X)^+\\)l}n+[)h1Y)V0_)l}n,Y)k-[*f)m}m-Y)v)g)m}m-Y)u*f)n}m-Y)t*g)n}m-Y)s*g)o}m-Y)k)p)o}m-X)l)_)d)p}m-X)l)_)d)p}m-X)l)_)c)q}m-X)l)_)c)q}m-X)k)`)b)r}m-X)k)`)b)r}m-X)k+^)a)s}m-X)k)V*])a)s}m-X*i)V)V)])`)t}m-X)V)h)V)V)\\)a)t}m-X*V*e)V)V)])`)u}m-X)V)W*b)V)W)])],u}m.W)W*V+^*V)W)^)Z+y}l/X)X)X1W)W)_)X*X/r}l/X)Y+^*W)`)W)W+\\)q}j1Y)[1X)`)W)V*`)p}j3W)f)a)V)V)c)o}j3X)d)b)V)V)d)n}j3Y)`+c)V)V)e)m}yV*[+f*W)f)l}yX.n*e)k}yy-_*k}yyZ2m}~~~",
    true,
    DustSearchMode::SecondToLastFrame
);

const DUST_WHIMSALOT: DustData = DustData::new_kill(
    "yk}p)Y)n}h+Y+W+Y+f}g-Y.Y-e}f/Y,Y/d}e,W+Y*Z*W,c}e,X*Y*Y+X+c}e+Y*V0V+X+c}l7i}W)i7g)V}V)i8g)V}V)h:f)V}W,c+X/X+c,V}W*V*b*d+_+V+V}W*W-]*a)Y*],W*W}X*Y,[*Y)\\+W*[,Y*W}Y*Y-Y*W+]*W*X-Z*X}Y+V.Z*W*Y)X)Y*Y1Z}[+V*\\*Y)X)]*\\*V*[}h*\\+\\*g}i)\\,[)h}j)[,Z)i}r)V)o}yk}g,c)V+f}e*Z*^+Y*d}c*[)W,V/[)c}b)\\)Y+V,X)[)b}b)[)Z)\\,\\)a}b)Y*V+W+V0\\)Y.W}c,W1V)W-])Z,W}i1V)W.\\)Z,W}i1V1\\)Z,W}e)W2V2V)Z)X)X)W}f4V3[)W)Y)W}g)V*V.W/V)\\)V)]}j)V*V)Y*V)V*]*_}l)k*_}y`)_}n)[)[)Z)a}n)[)[)Y)b}n)[)[,d}n)[)[,d}l+[*Z,d}k+\\,X.b}i-],j}i+`+j}yk}~~~",
    false,
    DustSearchMode::LastFrame
);

const DUST_FINAL_FROGGIT: DustData = DustData::new_kill(
    "yc}n)m}m+l}l-k}i)W-W)h}i*V-V*h}i3h}a)\\3[)a}`*X)X3W)X*`}`*X*Y/X*X*`}_,V+c+V,_}_1a1_}^2a2^}^+Z+_,Y+^}]+\\+],[+]}]*],[,]*]}]*^,Y,^*]}]*^,Y,^*]}]*^-W-^*]}X)Y*Y.V+W+V.Y*Y)X}X)Y*W0X,X0W*Y)X}X*X5W,W5X*X}X+XHX+X}XL0X}X7V-W-V7X}Y2V+V,Y,V+V2Y}V)W2k2W)V}V*W/n0W*V}W2W)V*V)_)V*V)V3W}W3V)V*V+[+V*V)V3W}XL0X}YL.Y}ZL,Z}\\L\\}^H^}c>c}yc}yc}Z)yX)Y}[*x*Z}[,Y+f+Y,Z}\\4b4[}]6\\6\\}_G^}b3X3a}\\)X1V*Z*V1X)[}]3X*X*X3\\}^2X/X2]}`E_}_4\\4^}_2`2^}_0d0^}_/Z)Z)Z/^}_/X-V-X/^}_.[)Z)[.^}`-[)Z)[-_}`-h-_}a-V)Y)X)Y)V-`}Z+Z.Z)V)Z.Z+Y}[4\\)\\4Z}\\2f2[}^-l-]}yc}yc}~~~",
    false,
    DustSearchMode::LastFrame
);

#[derive(Clone, Copy, PartialEq)]
#[expect(non_camel_case_types)]
pub enum Battlegroup {
    // TODO: various different orderings of encounters with multiple enemies
    Froggit,
    Whimsun,
    FroggitX_Whimsun,
    Moldsmal,
    MoldsmalX_Moldsmal_Moldsmal,
    FroggitX_Froggit,
    MoldsmalX_Moldsmal,
    //Moldsmal_Migosp,
    //Migosp_Vegetoid,
    //Loox,
    //Loox_Vegetoid,
    //Loox_Vegetoid_Migosp,
    //Vegetoid_Vegetoid,
    //Loox_Loox,
    //Vegetoid,
    Icecap_JerryS,
    Icecap_Snowdrake_JerryS,
    Aaron,
    Astigmatism,
    Madjick,
    KnightKnight,
    FinalFroggit_Astigmatism_WhimsalotX,
    KnightKnightX_Madjick,
    Whimsalot_AstigmatismX,
    Whimsalot_FinalFroggitX,
    FinalFroggit_AstigmatismX
}

impl Battlegroup {
    pub fn get_dust_config(&self) -> DustSearchConfig {
        match self {
            Self::Froggit =>
                DUST_FROGGIT.to_search_config(
                    216.0, 136.0,
                    "* YOU WON!* You earned 3 XP and 2 gold./%".len(),
                    "* YOU WON!* You earned 3 XP and 2 gold.* Your LOVE increased./%".len(),
                    Rect::new(211, 196, 47, 35),
                    false,
                    1
                ),
            Self::Whimsun =>
                DUST_WHIMSUN.to_search_config(
                    214.0, 16.0,
                    "* YOU WON!* You earned 2 XP and 2 gold./%".len(),
                    "* YOU WON!* You earned 2 XP and 2 gold.* Your LOVE increased./%".len(),
                    Rect::new(242, 60, 47, 35),
                    false,
                    1
                ),
            Self::FroggitX_Whimsun =>
                DUST_FROGGIT.to_search_config(
                    216.0, 136.0,
                    "* YOU WON!* You earned 5 XP and 4 gold./%".len(),
                    "* YOU WON!* You earned 5 XP and 4 gold.* Your LOVE increased./%".len(),
                    Rect::new(211, 196, 47, 35),
                    false,
                    2
                ),
            Self::Moldsmal =>
                DUST_MOLDSMAL.to_search_config(
                    216.0, 156.0,
                    "* YOU WON!* You earned 3 XP and 3 gold./%".len(),
                    "* YOU WON!* You earned 3 XP and 3 gold.* Your LOVE increased./%".len(),
                    Rect::new(241, 183, 47, 35),
                    false,
                    1
                ),
            Self::MoldsmalX_Moldsmal_Moldsmal =>
                DUST_MOLDSMAL.to_search_config(
                    15.0, 156.0,
                    "* YOU WON!* You earned 9 XP and 9 gold./%".len(),
                    "* YOU WON!* You earned 9 XP and 9 gold.* Your LOVE increased./%".len(),
                    Rect::new(41, 183, 47, 35),
                    false,
                    3
                ),
            Self::FroggitX_Froggit =>
                DUST_FROGGIT.to_search_config(
                    116.0, 136.0,
                    "* YOU WON!* You earned 6 XP and 4 gold./%".len(),
                    "* YOU WON!* You earned 6 XP and 4 gold.* Your LOVE increased./%".len(),
                    Rect::new(111, 196, 47, 35),
                    false,
                    2
                ),
            Self::MoldsmalX_Moldsmal =>
                DUST_MOLDSMAL.to_search_config(
                    116.0, 156.0,
                    "* YOU WON!* You earned 6 XP and 6 gold./%".len(),
                    "* YOU WON!* You earned 6 XP and 6 gold.* Your LOVE increased./%".len(),
                    Rect::new(141, 183, 47, 35),
                    false,
                    2
                ),
            Self::Icecap_JerryS =>
                DUST_JERRY.to_search_config(
                    216.0, 127.0,
                    "* YOU WON!* You earned 17 XP and 17 gold./%".len(),
                    "* YOU WON!* You earned 17 XP and 17 gold.* Your LOVE increased./%".len(),
                    Rect::new(210, 110, 220, 165),
                    false,
                    1),
            Self::Icecap_Snowdrake_JerryS =>
                DUST_JERRY.to_search_config(
                    216.0, 127.0,
                    "* YOU WON!* You earned 39 XP and 35 gold./%".len(),
                    "* YOU WON!* You earned 39 XP and 35 gold.* Your LOVE increased./%".len(),
                    Rect::new(210, 110, 220, 165),
                    false,
                    2),
            Self::Aaron =>
                DUST_AARON.to_search_config(
                    216.0, 38.0,
                    "* YOU WON!* You earned 52 XP and 25 gold./%".len(),
                    "* YOU WON!* You earned 52 XP and 25 gold.* Your LOVE increased./%".len(),
                    Rect::new(214, 187, 47, 35),
                    true,
                    1),
            Self::Astigmatism => 
                DUST_ASTIGMATISM.to_search_config(
                    228.0, 120.0,
                    "* YOU WON!* You earned 130 XP and 85 gold./%".len(),
                    "* YOU WON!* You earned 130 XP and 85 gold.* Your LOVE increased./%".len(),
                    Rect::new(250, 180, 47, 35),
                    true,
                    1
                ),
            Self::Madjick => 
                DUST_MADJICK.to_search_config(
                    244.0, 50.0,
                    "* YOU WON!* You earned 150 XP and 120 gold./%".len(),
                    "* YOU WON!* You earned 150 XP and 120 gold.* Your LOVE increased./%".len(),
                    Rect::new(251, 152, 47, 35),
                    true,
                    1
                ),
            Self::KnightKnight => 
                DUST_KNIGHT_KNIGHT.to_search_config(
                    56.0, 40.0,
                    "* YOU WON!* You earned 180 XP and 150 gold./%".len(),
                    "* YOU WON!* You earned 180 XP and 150 gold.* Your LOVE increased./%".len(),
                    Rect::new(170, 182, 47, 35),
                    true,
                    1
                ),
            Self::FinalFroggit_Astigmatism_WhimsalotX => 
                DUST_WHIMSALOT.to_search_config(
                    420.0, 110.0,
                    "* YOU WON!* You earned 360 XP and 245 gold./%".len(),
                    "* YOU WON!* You earned 360 XP and 245 gold.* Your LOVE increased./%".len(),
                    Rect::new(466, 152, 47, 35),
                    true,
                    3
                ),
            Self::KnightKnightX_Madjick => 
                DUST_KNIGHT_KNIGHT.to_search_config(
                    16.0, 50.0,
                    "* YOU WON!* You earned 330 XP and 270 gold./%".len(),
                    "* YOU WON!* You earned 330 XP and 270 gold.* Your LOVE increased./%".len(),
                    Rect::new(130, 192, 47, 35),
                    true,
                    2
                ),
            Self::Whimsalot_AstigmatismX => 
                DUST_ASTIGMATISM.to_search_config(
                    426.0, 120.0,
                    "* YOU WON!* You earned 240 XP and 165 gold./%".len(),
                    "* YOU WON!* You earned 240 XP and 165 gold.* Your LOVE increased./%".len(),
                    Rect::new(448, 180, 47, 35),
                    true,
                    2
                ),
            Self::Whimsalot_FinalFroggitX => 
                DUST_FINAL_FROGGIT.to_search_config(
                    426.0, 120.0,
                    "* YOU WON!* You earned 230 XP and 160 gold./%".len(),
                    "* YOU WON!* You earned 230 XP and 160 gold.* Your LOVE increased./%".len(),
                    Rect::new(458, 190, 47, 35),
                    true,
                    2
                ),
            Self::FinalFroggit_AstigmatismX => 
                DUST_ASTIGMATISM.to_search_config(
                    426.0, 120.0,
                    "* YOU WON!* You earned 250 XP and 165 gold./%".len(),
                    "* YOU WON!* You earned 250 XP and 165 gold.* Your LOVE increased./%".len(),
                    Rect::new(448, 180, 47, 35),
                    true,
                    2
                )
        }
    }
    pub fn get_name(&self) -> &'static str {
        match self {
            Self::Froggit => "Froggit",
            Self::Whimsun => "Whimsun",
            Self::FroggitX_Whimsun => "Froggit (X),\nWhimsun",
            Self::Moldsmal => "Moldsmal",
            Self::MoldsmalX_Moldsmal_Moldsmal => "Moldsmal (X),\nMoldsmal,\nMoldsmal",
            Self::FroggitX_Froggit => "Froggit (X),\nFroggit",
            Self::MoldsmalX_Moldsmal => "Moldsmal (X),\nMoldsmal",
            //Self::Moldsmal_Migosp => "Moldsmal,\nMigosp",
            //Self::Migosp_Vegetoid => "Migosp,\nVegetoid",
            //Self::Loox => "Loox",
            //Self::Loox_Vegetoid => "Loox,\nVegetoid",
            //Self::Loox_Vegetoid_Migosp => "Loox,\nVegetoid,\nMigosp",
            //Self::Vegetoid_Vegetoid => "Vegetoid,\nVegetoid",
            //Self::Loox_Loox => "Loox,\nLoox",
            //Self::Vegetoid => "Vegetoid",
            Self::Icecap_JerryS => "\nIce Cap,\nJerry (S)",
            Self::Icecap_Snowdrake_JerryS => "\nIce Cap,\nChilldrake,\nJerry (S)",
            Self::Aaron => "Aaron",
            Self::Astigmatism => "Astigmatism",
            Self::Madjick => "Madjick",
            Self::KnightKnight => "Knight Knight",
            Self::FinalFroggit_Astigmatism_WhimsalotX => "Final Froggit,\nAstigmatism,\nWhimsalot (X)",
            Self::KnightKnightX_Madjick => "Knight Knight (X),\nMadjick",
            Self::Whimsalot_AstigmatismX => "Whimsalot,\nAstigmatism (X)",
            Self::Whimsalot_FinalFroggitX => "Whimsalot,\nFinal Froggit (X)",
            Self::FinalFroggit_AstigmatismX => "Final Froggit,\nAstigmatism (X)"
        }
    }
} 

#[derive(Clone, Copy, PartialEq)]
pub enum Encounterer {
    Ruins1,
    Ruins3,
    Jerry,
    //Water2,
    Core
}

pub const GML_EPSILON: f64 = 0.00001;

fn steps_calculation(prng: &PrecomputedRNG, position: usize, base_amount: u32, random_amount: u32, population: u32, kills: u32) -> f64 {
    let base_amount = base_amount as f64;
    let random_amount = random_amount as f64;
    let population = population as f64;
    let kills = kills as f64;
    if kills >= population {
        return (base_amount + f64::round_ties_even(random_amount / 2.0)) * 5.0;
    }
    let mut population_factor = population / (population - kills);
    if population_factor >= (8.0 + GML_EPSILON) {
        population_factor = 8.0;
    }
    (base_amount + f64::round_ties_even(prng.get_f64(random_amount, position))) * population_factor
}

impl Encounterer {
    pub fn get_name(&self) -> &'static str {
        match self {
            Encounterer::Ruins1 => "Ruins1",
            Encounterer::Ruins3 => "Ruins3",
            Encounterer::Jerry => "Jerry",
            //Encounterer::Water2 => "Water2",
            Encounterer::Core => "Core"
        }
    }
    pub fn get_battlegroup_at_pos(&self, prng: &PrecomputedRNG, position: usize) -> Battlegroup {
        match self {
            Encounterer::Ruins1 => {
                let rng = prng.get_f64(2.0, position);
                if rng >= (1.0 + GML_EPSILON) {
                    Battlegroup::Whimsun
                } else {
                    Battlegroup::Froggit
                }
            },
            Encounterer::Ruins3 => {
                let rng = prng.get_f64(20.0, position);
                if rng >= (18.0 + GML_EPSILON) {
                    Battlegroup::MoldsmalX_Moldsmal
                } else if rng >= (15.0 + GML_EPSILON) {
                    Battlegroup::FroggitX_Froggit
                } else if rng >= (10.0 + GML_EPSILON) {
                    Battlegroup::MoldsmalX_Moldsmal_Moldsmal
                } else if rng >= (5.0 + GML_EPSILON) {
                    Battlegroup::Moldsmal
                } else {
                    Battlegroup::FroggitX_Whimsun
                }
            },
            Encounterer::Jerry => {
                let rng = f64::round_ties_even(prng.get_f64(15.0, position)) as u32;
                match rng {
                    0..9 => Battlegroup::Icecap_JerryS,
                    9.. => Battlegroup::Icecap_Snowdrake_JerryS
                }
            },
            // Encounterer::Water2 => {
            //     let rng = f64::floor(prng.get_f64(15.0, position)) as u32;
            //     match rng {
            //         0..4 => Battlegroup::Temmie,
            //         4..10 => Battlegroup::Woshua_Moldbygg,
            //         10.. => Battlegroup::Woshua_Aaron
            //     }
            // },
            Encounterer::Core => {
                let rng = f64::floor(prng.get_f64(15.0, position)) as u32;
                match rng {
                    0 => Battlegroup::Madjick,
                    1 => Battlegroup::KnightKnight,
                    2..4 => Battlegroup::FinalFroggit_Astigmatism_WhimsalotX,
                    4..7 => Battlegroup::KnightKnightX_Madjick,
                    7..10 => Battlegroup::Whimsalot_AstigmatismX,
                    10..13 => Battlegroup::Whimsalot_FinalFroggitX,
                    13.. => Battlegroup::FinalFroggit_AstigmatismX
                }
            }
        }
    }
    pub fn get_step_count_room_start(&self, prng: &PrecomputedRNG, position: usize, kills: u32) -> f64 {
        match self {
            Encounterer::Ruins1 => {
                steps_calculation(prng, position, 80, 40, 20, kills)
            },
            Encounterer::Ruins3 => {
                steps_calculation(prng, position, 60, 60, 20, kills)
            },
            Encounterer::Jerry => {
                // TODO: doesn't account for room_tundra_snowpuzz, or its associated flags
                steps_calculation(prng, position, 120, 30, 16, kills)
            },
            Encounterer::Core => todo!()
        }
    }
    pub fn get_step_count_same_room(&self, prng: &PrecomputedRNG, position: usize, kills_before_last_battle: u32) -> f64 {
        match self {
            Encounterer::Ruins1 => {
                steps_calculation(prng, position, 190, 80, 20, kills_before_last_battle)
            },
            Encounterer::Ruins3 => {
                steps_calculation(prng, position, 290, 100, 20, kills_before_last_battle)
            },
            Encounterer::Jerry => {
                steps_calculation(prng, position, 840, 680, 16, kills_before_last_battle)
            },
            Encounterer::Core => todo!()
        }
    }
    pub fn cycle_random_battlegroups(&self, battlegroup: Battlegroup) -> Battlegroup {
        match self {
            Encounterer::Ruins1 => {
                match battlegroup {
                    Battlegroup::Froggit => Battlegroup::Whimsun,
                    _ => Battlegroup::Froggit
                }
            },
            Encounterer::Ruins3 => {
                match battlegroup {
                    Battlegroup::FroggitX_Whimsun => Battlegroup::Moldsmal,
                    Battlegroup::Moldsmal => Battlegroup::MoldsmalX_Moldsmal_Moldsmal,
                    Battlegroup::MoldsmalX_Moldsmal_Moldsmal => Battlegroup::FroggitX_Froggit,
                    Battlegroup::FroggitX_Froggit => Battlegroup::MoldsmalX_Moldsmal,
                    _ => Battlegroup::FroggitX_Whimsun
                }
            },
            Encounterer::Jerry => {
                match battlegroup {
                    Battlegroup::Icecap_JerryS => Battlegroup::Icecap_Snowdrake_JerryS,
                    _ => Battlegroup::Icecap_JerryS
                }
            }
            Encounterer::Core => {
                match battlegroup {
                    Battlegroup::Madjick => Battlegroup::KnightKnight,
                    Battlegroup::KnightKnight => Battlegroup::KnightKnightX_Madjick,
                    Battlegroup::KnightKnightX_Madjick => Battlegroup::Whimsalot_AstigmatismX,
                    Battlegroup::Whimsalot_AstigmatismX => Battlegroup::Whimsalot_FinalFroggitX,
                    Battlegroup::Whimsalot_FinalFroggitX => Battlegroup::FinalFroggit_AstigmatismX,
                    Battlegroup::FinalFroggit_AstigmatismX => Battlegroup::FinalFroggit_Astigmatism_WhimsalotX,
                    Battlegroup::FinalFroggit_Astigmatism_WhimsalotX => Battlegroup::Madjick,
                    _ => Battlegroup::Madjick
                }
            }
        }
    }
}
