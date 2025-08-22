use sdl3::rect::Rect;

use crate::{compute_dust_search::DustSearchMode, dust::{DustData, DustSearchConfig}, frame_images::{FourPixelConfig, ImagePoint}, rng::PrecomputedRNG};

const FOURPIX_21_WHIMSALOT: FourPixelConfig = FourPixelConfig {
    pixel_coord_1_1: ImagePoint { x: 238, y: 314 },
    pixel_coord_1_1_size: 2,
    pixel_coord_2_1: ImagePoint { x: 238, y: 314 },
    pixel_coord_2_1_size: 2,
    pixel_coord_1_2: ImagePoint { x: 474, y: 155 },
    pixel_coord_1_2_size: 30,
    pixel_coord_2_2: ImagePoint { x: 474, y: 155 },
    pixel_coord_2_2_size: 30,
    pixel_match_color_1_1: 0xFF006AFF, // "RIGHT"
    pixel_match_color_1_2: 0xFF00007F, // too late
    pixel_match_color_2_1: 0xFF0000FF, // too early
    pixel_match_color_2_2: 0xFF21FF00, // "CLICK",
    pixel_replace_color_1_1: |col| match col {
        0xFFFFFFFF => 0xFF000000,
        _ => 0xFFFFFFFF
    },
    pixel_replace_color_1_2: |col| match col {
        0xFF000000 => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_1: |col| match col {
        0xFF000000 => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_2: |col| match col {
        0xFF000000 => 0xFF21FF00,
        _ => 0xFF000000
    }
};

// TODO: incomplete data
const FOURPIX_22_FROGGIT: FourPixelConfig = FourPixelConfig {
    pixel_coord_1_1: ImagePoint { x: 238, y: 314 },
    pixel_coord_1_1_size: 2,
    pixel_coord_2_1: ImagePoint { x: 238, y: 314 },
    pixel_coord_2_1_size: 2,
    pixel_coord_1_2: ImagePoint { x: 474, y: 155 },
    pixel_coord_1_2_size: 30,
    pixel_coord_2_2: ImagePoint { x: 230, y: 200 },
    pixel_coord_2_2_size: 30,
    pixel_match_color_1_1: 0xFF006AFF, // "RIGHT"
    pixel_match_color_1_2: 0xFF00007F, // too late
    pixel_match_color_2_1: 0xFF0000FF, // too early
    pixel_match_color_2_2: 0xFF21FF00, // "CLICK",
    pixel_replace_color_1_1: |col| match col {
        0xFFFFFFFF => 0xFF000000,
        _ => 0xFFFFFFFF
    },
    pixel_replace_color_1_2: |col| match col {
        0xFF000000 => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_1: |col| match col {
        0xFF000000 => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_2: |col| match col {
        0xFFFFFFFF => 0xFF21FF00,
        _ => 0xFF000000
    }
};

const FOURPIX_25_FINAL_FROGGIT: FourPixelConfig = FourPixelConfig {
    pixel_coord_1_1: ImagePoint { x: 295, y: 320 },
    pixel_coord_1_1_size: 2,
    pixel_coord_2_1: ImagePoint { x: 295, y: 320 },
    pixel_coord_2_1_size: 2,
    pixel_coord_1_2: ImagePoint { x: 484, y: 204 },
    pixel_coord_1_2_size: 30,
    pixel_coord_2_2: ImagePoint { x: 484, y: 204 },
    pixel_coord_2_2_size: 30,
    pixel_match_color_1_1: 0xFF006AFF, // "RIGHT"
    pixel_match_color_1_2: 0xFF00007F, // too late
    pixel_match_color_2_1: 0xFF0000FF, // too early
    pixel_match_color_2_2: 0xFF21FF00, // "CLICK",
    pixel_replace_color_1_1: |col| match col {
        0xFFFFFFFF => 0xFF000000,
        _ => 0xFFFFFFFF
    },
    pixel_replace_color_1_2: |col| match col {
        0xFF000000 => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_1: |col| match col {
        0xFF000000 => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_2: |col| match col {
        0xFF000000 => 0xFF21FF00,
        _ => 0xFF000000
    }
};

const FOURPIX_28_MADJICK_SOLO: FourPixelConfig = FourPixelConfig {
    pixel_coord_1_1: ImagePoint { x: 342, y: 318 },
    pixel_coord_1_1_size: 2,
    pixel_coord_2_1: ImagePoint { x: 342, y: 318 },
    pixel_coord_2_1_size: 2,
    pixel_coord_1_2: ImagePoint { x: 251, y: 153 },
    pixel_coord_1_2_size: 50,
    pixel_coord_2_2: ImagePoint { x: 251, y: 153 },
    pixel_coord_2_2_size: 50,
    pixel_match_color_1_1: 0xFF006AFF, // "RIGHT"
    pixel_match_color_1_2: 0xFF00007F, // too late
    pixel_match_color_2_1: 0xFF0000FF, // too early
    pixel_match_color_2_2: 0xFF21FF00, // "CLICK",
    pixel_replace_color_1_1: |col| match col {
        0xFFFFFFFF => 0xFF000000,
        _ => 0xFFFFFFFF
    },
    pixel_replace_color_1_2: |col| match col {
        0xFF000000 => 0xFF0000FF,
        0xFF494949 => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_1: |col| match col {
        0xFF000000 => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_2: |col| match col {
        0xFF000000 => 0xFF21FF00,
        0xFF494949 => 0xFF21FF00,
        _ => 0xFF000000
    }
};

const FOURPIX_23_130XP: FourPixelConfig = FourPixelConfig {
    pixel_coord_1_1: ImagePoint { x: 266, y: 318 },
    pixel_coord_1_1_size: 2,
    pixel_coord_2_1: ImagePoint { x: 266, y: 318 },
    pixel_coord_2_1_size: 2,
    pixel_coord_1_2: ImagePoint { x: 278, y: 314 },
    pixel_coord_1_2_size: 2,
    pixel_coord_2_2: ImagePoint { x: 278, y: 314 },
    pixel_coord_2_2_size: 2,
    pixel_match_color_1_1: 0xFF006AFF, // "RIGHT"
    pixel_match_color_1_2: 0xFF00007F, // too late
    pixel_match_color_2_1: 0xFF0000FF, // too early
    pixel_match_color_2_2: 0xFF21FF00, // "CLICK",
    pixel_replace_color_1_1: |col| match col {
        0xFFFFFFFF => 0xFF000000,
        _ => 0xFFFFFFFF
    },
    pixel_replace_color_1_2: |col| match col {
        0xFFFFFFFF => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_1: |col| match col {
        0xFF000000 => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_2: |col| match col {
        0xFFFFFFFF => 0xFF21FF00,
        _ => 0xFF000000
    }
};

const FOURPIX_24_240XP: FourPixelConfig = FourPixelConfig {
    pixel_coord_1_1: ImagePoint { x: 262, y: 314 },
    pixel_coord_1_1_size: 2,
    pixel_coord_2_1: ImagePoint { x: 262, y: 314 },
    pixel_coord_2_1_size: 2,
    pixel_coord_1_2: ImagePoint { x: 286, y: 314 },
    pixel_coord_1_2_size: 2,
    pixel_coord_2_2: ImagePoint { x: 286, y: 314 },
    pixel_coord_2_2_size: 2,
    pixel_match_color_1_1: 0xFF006AFF, // "RIGHT"
    pixel_match_color_1_2: 0xFF00007F, // too late
    pixel_match_color_2_1: 0xFF0000FF, // too early
    pixel_match_color_2_2: 0xFF21FF00, // "CLICK",
    pixel_replace_color_1_1: |col| match col {
        0xFFFFFFFF => 0xFF000000,
        _ => 0xFFFFFFFF
    },
    pixel_replace_color_1_2: |col| match col {
        0xFFFFFFFF => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_1: |col| match col {
        0xFF000000 => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_2: |col| match col {
        0xFFFFFFFF => 0xFF21FF00,
        _ => 0xFF000000
    }
};

const FOURPIX_24_250XP: FourPixelConfig = FourPixelConfig {
    pixel_coord_1_1: ImagePoint { x: 262, y: 314 },
    pixel_coord_1_1_size: 2,
    pixel_coord_2_1: ImagePoint { x: 262, y: 314 },
    pixel_coord_2_1_size: 2,
    pixel_coord_1_2: ImagePoint { x: 278, y: 312 },
    pixel_coord_1_2_size: 2,
    pixel_coord_2_2: ImagePoint { x: 278, y: 312 },
    pixel_coord_2_2_size: 2,
    pixel_match_color_1_1: 0xFF006AFF, // "RIGHT"
    pixel_match_color_1_2: 0xFF00007F, // too late
    pixel_match_color_2_1: 0xFF0000FF, // too early
    pixel_match_color_2_2: 0xFF21FF00, // "CLICK",
    pixel_replace_color_1_1: |col| match col {
        0xFFFFFFFF => 0xFF000000,
        _ => 0xFFFFFFFF
    },
    pixel_replace_color_1_2: |col| match col {
        0xFFFFFFFF => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_1: |col| match col {
        0xFF000000 => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_2: |col| match col {
        0xFFFFFFFF => 0xFF21FF00,
        _ => 0xFF000000
    }
};

const FOURPIX_34_3XP_150G: FourPixelConfig = FourPixelConfig {
    pixel_coord_1_1: ImagePoint { x: 442, y: 313 },
    pixel_coord_1_1_size: 2,
    pixel_coord_2_1: ImagePoint { x: 442, y: 313 },
    pixel_coord_2_1_size: 2,
    pixel_coord_1_2: ImagePoint { x: 454, y: 311 },
    pixel_coord_1_2_size: 2,
    pixel_coord_2_2: ImagePoint { x: 454, y: 311 },
    pixel_coord_2_2_size: 2,
    pixel_match_color_1_1: 0xFF006AFF, // "RIGHT"
    pixel_match_color_1_2: 0xFF00007F, // too late
    pixel_match_color_2_1: 0xFF0000FF, // too early
    pixel_match_color_2_2: 0xFF21FF00, // "CLICK",
    pixel_replace_color_1_1: |col| match col {
        0xFFFFFFFF => 0xFF000000,
        _ => 0xFFFFFFFF
    },
    pixel_replace_color_1_2: |col| match col {
        0xFFFFFFFF => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_1: |col| match col {
        0xFF000000 => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_2: |col| match col {
        0xFFFFFFFF => 0xFF21FF00,
        _ => 0xFF000000
    }
};

const FOURPIX_34_3XP_270G: FourPixelConfig = FourPixelConfig {
    pixel_coord_1_1: ImagePoint { x: 437, y: 313 },
    pixel_coord_1_1_size: 2,
    pixel_coord_2_1: ImagePoint { x: 437, y: 313 },
    pixel_coord_2_1_size: 2,
    pixel_coord_1_2: ImagePoint { x: 461, y: 311 },
    pixel_coord_1_2_size: 2,
    pixel_coord_2_2: ImagePoint { x: 461, y: 311 },
    pixel_coord_2_2_size: 2,
    pixel_match_color_1_1: 0xFF006AFF, // "RIGHT"
    pixel_match_color_1_2: 0xFF00007F, // too late
    pixel_match_color_2_1: 0xFF0000FF, // too early
    pixel_match_color_2_2: 0xFF21FF00, // "CLICK",
    pixel_replace_color_1_1: |col| match col {
        0xFFFFFFFF => 0xFF000000,
        _ => 0xFFFFFFFF
    },
    pixel_replace_color_1_2: |col| match col {
        0xFFFFFFFF => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_1: |col| match col {
        0xFF000000 => 0xFF0000FF,
        _ => 0xFF000000
    },
    pixel_replace_color_2_2: |col| match col {
        0xFFFFFFFF => 0xFF21FF00,
        _ => 0xFF000000
    }
};

// TODO: incomplete data
const DUST_FROGGIT: DustData = DustData::new(
    "ye}ye}ye}j7e}h+b+c}g*f*b}f*h*a}e*`+\\*`}e*_-[*`}d*`*W*[*_}d*\\)X/Z*_}d*\\*W-\\*_}d*\\1\\*_}d*\\2[*_}d*]1[*_}d*^/\\*_}d*l*_}d+Y)Z)Z)Z*_}e+W)V)X)V)X)V)X,^}].X,X+X+X-W)]}[2W*g*X)]}Z4W)g*X)]}Y6i*X)]}X8h*X)]}W:g*X)]}W:g*X)]}V*X7e+X)]}V*V)V7V)c*Y)X+W}V*V)V.X.V)b+Y)W-V}V*X.V)V.V*_,Z)W-V}V3V)V.V,[-[)W-V}V3X.W4])X+W}V.Y1Y1_)Y)X}W,V,V0l)X)Y}W,V,V/m)Y)X}X,Y/n)X)Y}Y5o+W)X}Z3p)W*Y}\\/r)]}y\\)]}c)q)]}d)o)^}d)o)^}e)m)_}c)W)k)`}c*W)i)a}c+W*e*W)_}c,X+_+X*_}d-Y2Y,_}d,g,`}d,g,`}a/d/`}`1b1_}_)V*Y*a)V*Y*_}_)V)[)a)V)[)_}ye}~~~",
    false,
    DustSearchMode::LastFrame
);

// TODO: incomplete data
const DUST_WHIMSUN: DustData = DustData::new(
    "ye}ye}ye}f*g)c}e)W)e)V)b}d)Y)d)V)b}c+X)c)X)a}c+X)c)X)a}j)Z.W)X*`}j)X1V)X+_}j)V4Y*`}j+X*X+f}j)Z*Y+e}i)Z,Y*e}h*Y-Z*d}g*Y.Z*d}g*X0Y*d}f<d}f+V-V.V,c}f0X2c}f0X2c}f/Z1c}b)X.\\0c}a*X.\\0c}`+W/V)W)W0c}_*V)W>W*_}]*X)W>W*_}[*V)W)W?V+_}Z+W+W-W.W/W*V)^}Y*V)W*W-Y,Y.W,^}W,V,X,Z,Y.W+V)]}X-W)X*W+W,V)W.X*V*\\}`+W6W-X-\\}c:W+X*X)[}b=X+W*V)[}b@[,Z}b?e}b)X*X+X0e}g)Y)Y*X*f}ye}ye}k)[)k}k)[)k}k)[)k}k)[)k}i+[+i}h+\\,h}g+^,g}g*a*g}ye}ye}ye}ye}~~~",
    false,
    DustSearchMode::LastFrame
);

// TODO: incomplete data
const DUST_MOLDSMAL: DustData = DustData::new(
    "yd}yd}yd}d.X+W,g}b*[)V)W)V)Y*e}a/W)V)V)V)W/c}h+W)W+j}_1\\)X1a}^)],\\+\\*_}])W/c.X)^}\\,p*W)]}[*X1_0W)W)\\}[)V+Y*c*Z*V)W)[}Z)V)Y*Y+W*Y)X+Y)V)V)[}Y)V)Y)W.W+V*W+X)Y)V)V)Z}Y)V)X)V*Y*V,V+W-V)Y)V)V)Y}X)V)X)V*Z)V*W)V)V*W-V)Y)V*Y}X)V)W)V*Z)V*X)V)W*V)W*V*V)V)V)V)X}W)V)X)V)[)V)Y)V)X)V)X*V)Y)V)Y}W)V)X)V)V*X)V)Y)V)X)V)Y*V)X)V)Y}W)V)W)V)W*X)V)Y)V)X)V)Y*V)V)W)V)X}X)X)V)V+X)V)V)W)V)X)V)V*W)V)V)W)V)X}X)W*V)V)Y)V)W)W)V)V)V)V)W)W)V)Z)Y}W)X)V*[)V)V*W)V)V)V)V)W*V*V)Y)Y}W)X)V)[)V)W*W)V)V)W)V)W)W)V)Y*X}W)X)V)W)X)V)W)X)V)V)W)V*V)W)V)Z)X}W)X)V)V)Y)V)[)V)V)X)V)Y*V)Y)X}X)V*V)V)Y)V)[)V)Z)V)Z)V)Y)X}X)V)V*V)X)W)V)X)V)[)V)W)W)V)X*X}X)V)V)[)V*V)X)V)[)V)W)X)V)W)Y}Y*V)[)V)W)X)V)W)X)V*Z)V+Z}\\)[)V)W)X)V)W)Y)V)[)]}\\)Z)V*[)V)W)Y)V)[)]}\\)Z)V)\\)V)W)Y)V)[)]}])Y)V)\\)V)\\)V)Y*^}^,W)\\)V)\\)W-_}d)\\)V)\\)f}e)[)V)[)g}f.X.h}yd}yd}yd}yd}~~~",
    false,
    DustSearchMode::LastFrame
);

// TODO: incomplete data
const DUST_MIGOSP: DustData = DustData::new(
    "ye}ye}b+f+e}a-d-d}_/c*V,c}^-W*b)X,b}]-Y)a*Y,a}]+[*`)[+a}g)`)\\*a}g*^*j}h)V/V)k}g)W/m}f*X-X)j}e,W-W+i}d:h}c<g}c<g}b*[.[*f}b*\\,\\*f}b*]*]*f}a+g+e}a,\\*\\,e}]+V@e}\\+V/W)V)V)V)W/V+a}[+W-d-W+`}[*X,V)b)V-W+_}[*X,V+V/V,V,X*_}[*X-X1Y-X*_}[*XBX*_}[-]1_,_}[,_/_-_}`-Z-Z.d}`-W)W-W)W.d}`.W)V-V)W/d}`Bd}a@e}a+f,e}a@e}b)i)f}b*g*f}c<g}d:h}ye}ye}ye}g*\\*l}g*\\*l}g*\\*l}c.\\.h}a0\\0f}`0^0e}ye}ye}~~~",
    false,
    DustSearchMode::LastFrame
);

// TODO: incomplete data
const DUST_LOOX: DustData = DustData::new(
    "ye}ye}w+d}a*j,b}`*l,a}_+l-`}^+n,`}],n-_}\\-n-_}\\-^1\\-_}\\-[7Y-_}\\-Y;W-_}\\.WC_}\\K_}\\6_3_}]2d0`}^/h.`}^.j-`}].l,`}]-m-_}]-n,_}\\-o-^}\\,`._-^}\\,^1_-]}[-]+[*^-]}[-]*])^-]}[-\\*_)].\\}[-\\*Y*Y)].\\}[-\\*X)W)X)].\\}Z.\\*X)W)X)].\\}Z.\\*X)W)X)]/[}Y/\\*Y*Y)\\0[}Y0\\*])]1Z}X1\\+[*]1Z}X2\\1]3Y}W3^.^+X-Y}W-Y+k*[-X}V-\\)k)],X}V,^)i)_,W}V+`)g)a+W}V+a*c*b+W}V+c6d+W}V+e2f+W}V+y\\+W}V+y\\,V}V.yX-V}V/yV.V}W.a)\\)e*V+V}X+V)a)V)X)W*c)W+V}X)V)a)V)V*V+V*Z)])W*W}X)V)e+V*])_*X}X)`)i)d}d)i)d}d)h*d}d)g*e}d*e+e}c,d.b}a.c0a}`0b0a}ye}ye}~~~",
    false,
    DustSearchMode::LastFrame
);

// TODO: incomplete data
const DUST_VEGETOID: DustData = DustData::new(
    "ye}m)p}n)o}i*X)W+j}e+X)W)V)Y+f}h)W*V)V)W*i}c-V)W)V)V)V)W-d}b)Z)V)V)V)V)W*Z)c}c,W)W)V)V)V)X,d}a*Y)W)V)V)V)X*Y*b}`)\\)V)V)V)V*V)])a}i)W)V*V*^)a}j)V*V*V*j}j)W)V*V)k}m)V*m}c+e+e}`1`0b}_G`}^I_}]J_}]?X1^}\\?Z0^}\\1Y1\\/^}\\0[0\\/^}\\0\\/\\/^}\\0\\/X3^}\\0\\/W4^}]3X/W4^}]4W/W3_}]4W<_}^3W<_}^H`}^H`}^6W2W,`}_,X.W*W.W,`}_-X,W+W-W,a}`,X2W+W-a}`-X6W,b}a,Y4W-b}a-X3X,c}b-X2W-c}c-W1W-d}c-X/X-d}d-Y*Z-e}d._,f}e-^-f}f-\\-g}f.X)V.g}g7h}h5i}i3j}j1k}k.m}ye}ye}~~~",
    false,
    DustSearchMode::LastFrame
);

const DUST_ASTIGMATISM: DustData = DustData::new(
    "ya}X*yW*X}W*yY*W}V+yY+V}V+yY+V}V+e0e+V}V,a6a,V}V-^:^-V}W.Y@Y.W}WL0W}XL.X}XL.X}Y9[9Y}Z8[8Z}\\6[6\\}]5[5]}]-Y-Y-Y-]}V)Z/X-Y-X/Z)V}V*X0X-Y-X0X*V}V5Y-W-Y5V}W5X-W-X5W}X4X-W-W5X}Y4W+[+W4Y}[3c3[}[1g1[}[/W*c*W/[}[.V+e+V.[}[-V+g+V-[}\\,V+g+V,\\}\\-V*g*V-\\}\\-V*g*V-\\}]-V*e*V-]}].V)e)V.]}^.V)c)V.^}^/e/^}_0a0_}])W2[2W)]}\\+W@W+\\}\\+X>X+\\}\\+Y<Y+\\}[,[8[+\\}[,^2^,[}[,Y)g)Y,[}Z+V)Y+c+Y,[}Z+V)Y0Y0Y)V+Z}Z*W)Y/[/Y)V+Z}Z*].[.]*Z}Z)^.[.^)Z}d-]-d}d-]-d}e,],e}e,],e}e,],e}f*_*f}f*_*f}g)_)g}g)_)g}ya}~~~",
    false,
    DustSearchMode::SecondToLastFrame
);

const DUST_MADJICK: DustData = DustData::new(
    "yy}n/yY}l*\\*yW}k)`)yV}k)a)y}j)c)x}j)Y,\\)w}j)X)Y)\\)v}j)X)W)W)\\)u}k)X*V)W)[)u}l*X)X)\\)t}n+Z)[)t}v)\\)s}v)\\)s}v)])g.Z}k)^)^)^)\\)[)Y}j)V)])^)])V)Z)V)[)X}j)W)[)`)[)W)Y)W)\\)W}j)X.b.X)Y)X*V)V)V)W}Y.a)r)Z)Z)V)V*W}X)[)`)r)Z)Y)Z)W}W)V)[)`)p)[)Z)Y)W}V)W)\\)`)n)])X)Y)X}V)X*V)V)V)a*j*_)X)W)Y}V)Z)V)V*c+e*b.Z}V)Y)Z)f,^+o}V)Z)Y)a)]1r}W)X)Y)b,y\\}X)X)W)c.yZ}Y.d,V-],Y)i}n,W<j}n+V)Y8k}o)Y*[1m}o+W+V*V)v}p+W*V*V)V)V)V)p}q+Y*V)V)V)r}r-yW}t3s}k.c.m}i*[*_*[*k}h)_)])_)j}g)a)[)a)i}g)X,[)Y)[,X)i}g)W)Y)[)W)[)Y)W)i}g)V)[)W*V)W)V*W)[)V)i}g)V)\\)V*W*W*V)\\)V)i}h*\\)V*W*W*V)\\*j}i*\\)Y*Y)\\*k}r)Y*Y)t}s)X)Y)u}t)W)X)v}s)X)Y)u}s)X*Y)t}r)Z)X)u}s)Y)Y)t}r)Z)Z)s}r)Z)Z)s}q)a)s}j)Z)c)Z)l}j)X*e*X)l}j,i,l}k)m)m}^*V)\\*V+h*V*]*V)_}])X)[*V)X;W*V)[*W*^}`)\\)X*j*W*[*W)^}])W)\\)Z)h)e)^}])^*[)f)[)_)^}`*Y*])c*])Z)X)^}]*X)V+[)V)e*\\+W)X*^}d)])h+]*Y)_}^)b)k*b)_}_*^*m)V)^*`}`*X)X)q*X)X*a}a+V)W)s)X-b}d*V)u)W)f}f*v)V)f}e*x*f}yf)g}yy}~~~",
    true,
    DustSearchMode::SecondToLastFrame
);

const DUST_KNIGHT_KNIGHT: DustData = DustData::new(
    "yyyX}o)yya}o*yy`}o+yy_}m)V+c)s)y}m)V+`,s-u}k/V)\\.a.a.t}k2Z,c+X*c,s}i6X+c*V+W*c+s}h6Y+b,Y,b+s}k2Z+`8`+s}k-V/W,],V2V,],s}g1V0V8W0W8s}h-V)W,[5V)X.X)V5t}k*W*V)V)\\2X)X.X)W3u}k*V)W)W,f)X*W*X)y^}k)W)W)V,f*a*y]}f.V)V*W*h+_,y\\}g0W)V*\\1W,_,W1u}k,V-[)_.X*X._)V/l}k,V-Z,\\0W*W0\\,V*Y,h}k2f0V*V0d)\\+e}i6V._8_/V)^)d}i6V1]6]3V)^*b}l0X4\\4\\5V)`)a}l0V7\\2\\7V)`)`}n,X8\\0\\8V)a)_}n,X9\\.\\9W)Z0^}i)Z*Y:]*]:W)W+\\*]}h)[*Y;e;W)V)a)\\}g)Y0W;c;X*j}f)Z0Y9c:b-^}e)],]8a8b+X)_}d)^,_6a6b*Z)`}c)_,V)_5_5b*[)a}b1Z*V*`4]4b*])a}a)\\)W,g3V)W)V3n)a}`)\\)W1^+Y3W3X+j)a}_)\\)W3[*^1W1]*i)`}_)\\)V4Z)b/W/a)h)`}^)\\)W._)e-W-d)g)`}^)\\)W3Y)h*Y*g)f)`}])\\)W4Y)y^)f)`}])\\)W4X)i*X*i)f)_}])\\)W.^)h)\\)h)f)_}])\\)W3Y)g)^)g)f)_}])\\)W3Y)e*X-X*e)f)_}])\\)X,`*a*X1X*u)_}\\)^)W1]4Y3Y,W+V*Z)c)_}\\)^)X0X)i*V-V*^+V+[,`)^}[)\\-c,a)W*W-W*W*X+V+W*\\*^)Y)Y}[)[)`*\\*W+V*V+V*V*V/V*V-V+V+V+V)\\*\\)X*Y}Z)[)a+^,V*V+V*W3W-V+V+V+V*]*Z)W)V)Y}Y)[)b+[.V+V+V*W3W-W*V+V+V*_)Z*W)Y}X)Y+c+\\,V,V*V+X+X+Y,^*V+`-Y)Y}V/f+a+V+V+X*Z*Y+V)`*i)Z}n+`,V+V*Y*Z*Y*V)w)Z}n+`,V+V*h)w)[}n+_,V,V*f)V)e)f)[}n+_+W+V+g)g)d)\\}n+_)Y+V+g)g)d)\\}n+d*W+g)g)c)]}n+^)Y+V+X)`)X)g)b)^}n+^)Y*W+X)`)X)f)V)`)_}n+])Z)W,W*X)X)X*W)f)V)V+Z*`}n+])]+X+W*V*W+W)e)W)Y-b}n+\\)]+Y+V+V+V+V*d)X)k}n+\\)e5W)d)Y)k}n+[)h1Y)c)Z)k}n+[)f)`)W)a*[)k}n+[)g*\\*X)^+\\)l}n+[)h1Y)V0_)l}n,Y)k-[*f)m}m-Y)v)g)m}m-Y)u*f)n}m-Y)t*g)n}m-Y)s*g)o}m-Y)k)p)o}m-X)l)_)d)p}m-X)l)_)d)p}m-X)l)_)c)q}m-X)l)_)c)q}m-X)k)`)b)r}m-X)k)`)b)r}m-X)k+^)a)s}m-X)k)V*])a)s}m-X*i)V)V)])`)t}m-X)V)h)V)V)\\)a)t}m-X*V*e)V)V)])`)u}m-X)V)W*b)V)W)])],u}m.W)W*V+^*V)W)^)Z+y}l/X)X)X1W)W)_)X*X/r}l/X)Y+^*W)`)W)W+\\)q}j1Y)[1X)`)W)V*`)p}j3W)f)a)V)V)c)o}j3X)d)b)V)V)d)n}j3Y)`+c)V)V)e)m}yV*[+f*W)f)l}yX.n*e)k}yy-_*k}yyZ2m}~~~",
    true,
    DustSearchMode::SecondToLastFrame
);

const DUST_WHIMSALOT: DustData = DustData::new(
    "yk}p)Y)n}h+Y+W+Y+f}g-Y.Y-e}f/Y,Y/d}e,W+Y*Z*W,c}e,X*Y*Y+X+c}e+Y*V0V+X+c}l7i}W)i7g)V}V)i8g)V}V)h:f)V}W,c+X/X+c,V}W*V*b*d+_+V+V}W*W-]*a)Y*],W*W}X*Y,[*Y)\\+W*[,Y*W}Y*Y-Y*W+]*W*X-Z*X}Y+V.Z*W*Y)X)Y*Y1Z}[+V*\\*Y)X)]*\\*V*[}h*\\+\\*g}i)\\,[)h}j)[,Z)i}r)V)o}yk}g,c)V+f}e*Z*^+Y*d}c*[)W,V/[)c}b)\\)Y+V,X)[)b}b)[)Z)\\,\\)a}b)Y*V+W+V0\\)Y.W}c,W1V)W-])Z,W}i1V)W.\\)Z,W}i1V1\\)Z,W}e)W2V2V)Z)X)X)W}f4V3[)W)Y)W}g)V*V.W/V)\\)V)]}j)V*V)Y*V)V*]*_}l)k*_}y`)_}n)[)[)Z)a}n)[)[)Y)b}n)[)[,d}n)[)[,d}l+[*Z,d}k+\\,X.b}i-],j}i+`+j}yk}~~~",
    false,
    DustSearchMode::LastFrame
);

const DUST_FINAL_FROGGIT: DustData = DustData::new(
    "yc}n)m}m+l}l-k}i)W-W)h}i*V-V*h}i3h}a)\\3[)a}`*X)X3W)X*`}`*X*Y/X*X*`}_,V+c+V,_}_1a1_}^2a2^}^+Z+_,Y+^}]+\\+],[+]}]*],[,]*]}]*^,Y,^*]}]*^,Y,^*]}]*^-W-^*]}X)Y*Y.V+W+V.Y*Y)X}X)Y*W0X,X0W*Y)X}X*X5W,W5X*X}X+XHX+X}XL0X}X7V-W-V7X}Y2V+V,Y,V+V2Y}V)W2k2W)V}V*W/n0W*V}W2W)V*V)_)V*V)V3W}W3V)V*V+[+V*V)V3W}XL0X}YL.Y}ZL,Z}\\L\\}^H^}c>c}yc}yc}Z)yX)Y}[*x*Z}[,Y+f+Y,Z}\\4b4[}]6\\6\\}_G^}b3X3a}\\)X1V*Z*V1X)[}]3X*X*X3\\}^2X/X2]}`E_}_4\\4^}_2`2^}_0d0^}_/Z)Z)Z/^}_/X-V-X/^}_.[)Z)[.^}`-[)Z)[-_}`-h-_}a-V)Y)X)Y)V-`}Z+Z.Z)V)Z.Z+Y}[4\\)\\4Z}\\2f2[}^-l-]}yc}yc}~~~",
    false,
    DustSearchMode::LastFrame
);

pub fn get_debug_search_config() -> DustSearchConfig {
    DUST_FROGGIT.to_search_config(0.0, 0.0, 20, 21, Rect::new(0, 0, 640, 480), FOURPIX_23_130XP)
}

#[derive(Clone, Copy, PartialEq)]
#[expect(non_camel_case_types)]
pub enum Battlegroup {
    // TODO: incomplete data
    Froggit,
    //Whimsun,
    //Froggit_Whimsun,
    //Moldsmal,
    //Moldsmal_Moldsmal_Moldsmal,
    //Froggit_Froggit,
    //Moldsmal_Moldsmal,
    //Moldsmal_Migosp,
    //Migosp_Vegetoid,
    //Loox,
    //Loox_Vegetoid,
    //Loox_Vegetoid_Migosp,
    //Vegetoid_Vegetoid,
    //Loox_Loox,
    //Vegetoid,
    Astigmatism,
    Madjick,
    KnightKnight,
    FinalFroggit_Astigmatism_Whimsalot,
    KnightKnight_Madjick,
    Whimsalot_Astigmatism,
    Whimsalot_FinalFroggit,
    FinalFroggit_Astigmatism
}

impl Battlegroup {
    pub fn get_dust_config(&self) -> DustSearchConfig {
        match self {
            // TODO: incomplete data
            Self::Froggit =>
                DUST_FROGGIT.to_search_config(
                    216.0, 136.0,
                    "* YOU WON!* You earned 3 XP and 2 gold./%".len(),
                    "* YOU WON!* You earned 3 XP and 2 gold.* Your LOVE increased./%".len(),
                    Rect::new(211, 207, 47, 35),
                    FOURPIX_23_130XP // TODO
                ),
            Self::Astigmatism => 
                DUST_ASTIGMATISM.to_search_config(
                    228.0, 120.0,
                    "* YOU WON!* You earned 130 XP and 85 gold./%".len(),
                    "* YOU WON!* You earned 130 XP and 85 gold.* Your LOVE increased./%".len(),
                    Rect::new(250, 180, 47, 35),
                    FOURPIX_23_130XP
                ),
            Self::Madjick => 
                DUST_MADJICK.to_search_config(
                    244.0, 50.0,
                    "* YOU WON!* You earned 150 XP and 120 gold./%".len(),
                    "* YOU WON!* You earned 150 XP and 120 gold.* Your LOVE increased./%".len(),
                    Rect::new(251, 152, 47, 35),
                    FOURPIX_28_MADJICK_SOLO
                ),
            Self::KnightKnight => 
                DUST_KNIGHT_KNIGHT.to_search_config(
                    56.0, 40.0,
                    "* YOU WON!* You earned 180 XP and 150 gold./%".len(),
                    "* YOU WON!* You earned 180 XP and 150 gold.* Your LOVE increased./%".len(),
                    Rect::new(170, 182, 47, 35),
                    FOURPIX_34_3XP_150G
                ),
            Self::FinalFroggit_Astigmatism_Whimsalot => 
                DUST_WHIMSALOT.to_search_config(
                    420.0, 110.0,
                    "* YOU WON!* You earned 360 XP and 245 gold./%".len(),
                    "* YOU WON!* You earned 360 XP and 245 gold.* Your LOVE increased./%".len(),
                    Rect::new(466, 152, 47, 35),
                    FOURPIX_21_WHIMSALOT
                ),
            // TODO: knight knight & madjick, but when finishing with madjick
            Self::KnightKnight_Madjick => 
                DUST_KNIGHT_KNIGHT.to_search_config(
                    16.0, 50.0,
                    "* YOU WON!* You earned 330 XP and 270 gold./%".len(),
                    "* YOU WON!* You earned 330 XP and 270 gold.* Your LOVE increased./%".len(),
                    Rect::new(130, 192, 47, 35),
                    FOURPIX_34_3XP_270G
                ),
            Self::Whimsalot_Astigmatism => 
                DUST_ASTIGMATISM.to_search_config(
                    426.0, 120.0,
                    "* YOU WON!* You earned 240 XP and 165 gold./%".len(),
                    "* YOU WON!* You earned 240 XP and 165 gold.* Your LOVE increased./%".len(),
                    Rect::new(448, 180, 47, 35),
                    FOURPIX_24_240XP
                ),
            Self::Whimsalot_FinalFroggit => 
                DUST_FINAL_FROGGIT.to_search_config(
                    426.0, 120.0,
                    "* YOU WON!* You earned 230 XP and 160 gold./%".len(),
                    "* YOU WON!* You earned 230 XP and 160 gold.* Your LOVE increased./%".len(),
                    Rect::new(458, 190, 47, 35),
                    FOURPIX_25_FINAL_FROGGIT
                ),
            Self::FinalFroggit_Astigmatism => 
                DUST_ASTIGMATISM.to_search_config(
                    426.0, 120.0,
                    "* YOU WON!* You earned 250 XP and 165 gold./%".len(),
                    "* YOU WON!* You earned 250 XP and 165 gold.* Your LOVE increased./%".len(),
                    Rect::new(448, 180, 47, 35),
                    FOURPIX_24_250XP
                )
        }
    }
    pub fn get_name(&self) -> &'static str {
        match self {
            Self::Froggit => "Froggit",
            // TODO: incomplete data
            //Self::Whimsun => "Whimsun",
            //Self::Froggit_Whimsun => "Froggit,\nWhimsun",
            //Self::Moldsmal => "Moldsmal",
            //Self::Moldsmal_Moldsmal_Moldsmal => "Moldsmal,\nMoldsmal,\nMoldsmal",
            //Self::Froggit_Froggit => "Froggit,\nFroggit",
            //Self::Moldsmal_Moldsmal => "Moldsmal,\nMoldsmal",
            //Self::Moldsmal_Migosp => "Moldsmal,\nMigosp",
            //Self::Migosp_Vegetoid => "Migosp,\nVegetoid",
            //Self::Loox => "Loox",
            //Self::Loox_Vegetoid => "Loox,\nVegetoid",
            //Self::Loox_Vegetoid_Migosp => "Loox,\nVegetoid,\nMigosp",
            //Self::Vegetoid_Vegetoid => "Vegetoid,\nVegetoid",
            //Self::Loox_Loox => "Loox,\nLoox",
            //Self::Vegetoid => "Vegetoid",
            Self::Astigmatism => "Astigmatism",
            Self::Madjick => "Madjick",
            Self::KnightKnight => "Knight Knight",
            Self::FinalFroggit_Astigmatism_Whimsalot => "Final Froggit,\nAstigmatism,\nWhimsalot",
            Self::KnightKnight_Madjick => "Knight Knight,\nMadjick",
            Self::Whimsalot_Astigmatism => "Whimsalot,\nAstigmatism",
            Self::Whimsalot_FinalFroggit => "Whimsalot,\nFinal Froggit",
            Self::FinalFroggit_Astigmatism => "Final Froggit,\nAstigmatism"
        }
    }
} 

#[derive(Clone, Copy, PartialEq)]
pub enum Encounterer {
    Core
}

impl Encounterer {
    pub fn get_battlegroup_at_pos(&self, prng: &PrecomputedRNG, position: usize) -> Battlegroup {
        match self {
            Encounterer::Core => {
                let rng = f64::floor(prng.get_f64(15.0, position)) as u32;
                match rng {
                    0 => Battlegroup::Madjick,
                    1 => Battlegroup::KnightKnight,
                    2..4 => Battlegroup::FinalFroggit_Astigmatism_Whimsalot,
                    4..7 => Battlegroup::KnightKnight_Madjick,
                    7..10 => Battlegroup::Whimsalot_Astigmatism,
                    10..13 => Battlegroup::Whimsalot_FinalFroggit,
                    13.. => Battlegroup::FinalFroggit_Astigmatism
                }
            }
        }
    }
    pub fn cycle_random_battlegroups(&self, battlegroup: Battlegroup) -> Battlegroup {
        match self {
            Encounterer::Core => {
                match battlegroup {
                    Battlegroup::Madjick => Battlegroup::KnightKnight,
                    Battlegroup::KnightKnight => Battlegroup::KnightKnight_Madjick,
                    Battlegroup::KnightKnight_Madjick => Battlegroup::Whimsalot_Astigmatism,
                    Battlegroup::Whimsalot_Astigmatism => Battlegroup::Whimsalot_FinalFroggit,
                    Battlegroup::Whimsalot_FinalFroggit => Battlegroup::FinalFroggit_Astigmatism,
                    Battlegroup::FinalFroggit_Astigmatism => Battlegroup::FinalFroggit_Astigmatism_Whimsalot,
                    Battlegroup::FinalFroggit_Astigmatism_Whimsalot => Battlegroup::Madjick,
                    _ => Battlegroup::Madjick
                }
            }
        }
    }
}