use std::fs::File;
use clap::{Arg, App};

// TODO: should be a bit object-oriented or something, instead of globals
static mut MINX:f32 = std::f32::MAX;
static mut MAXX:f32 = std::f32::MIN;

// return the min of two f32s
fn f32min(a: f32, b:f32) -> f32 {
    if a < b { return a; }
    else     { return b; }
}

// return the max of two f32s
fn f32max(a: f32, b:f32) -> f32 {
    if a > b { return a; }
    else     { return b; }
}

// convert x from the x range in the flat model into the corresponding angle on the cylindrical model
unsafe fn x2angle(x: f32) -> f32 {
    let k = (x - MINX) / (MAXX - MINX); // ranges from 0 to 1
    return k * std::f32::consts::PI / 2.0;
}

// wrap (x,y) from the flat model onto the cylindrical model
unsafe fn wrapxy(x: f32, y: f32) -> (f32, f32) {
    let angle = x2angle(x);
    let radius = y;
    return (radius * angle.cos(), radius * angle.sin());
}

unsafe fn wrapvertex(v: [f32;3]) -> [f32;3] {
    let (x,y) = wrapxy(v[0], v[1]);
    return [x,y,v[2]];
}

fn main() { unsafe {
    let matches = App::new("stlwrap")
        .version("0.1.0")
        .author("James Stanley <james@incoherency.co.uk>")
        .about("Wrap an STL file into a cylinder")
        .arg(Arg::with_name("FILE")
                 .index(1)
                 .required(true)
                 .help("A cool file"))
        .arg(Arg::with_name("maxlength")
                 .short("m")
                 .long("number")
                 .takes_value(true)
                 .help("Maximum length of triangle sides before wrapping"))
        .get_matches();

    let stlfilename = matches.value_of("FILE").unwrap_or("input.txt");

    let maxlength_s = matches.value_of("maxlength").unwrap_or("1");
    let maxlength = maxlength_s.parse::<f32>().unwrap();

    println!("Max. side length is {}", maxlength);

    // read in the stl file
    let mut file = File::open(stlfilename).unwrap();
    let stl = stl::read_stl(&mut file).unwrap();

    // TODO: subdivide triangles with long sides so that all sides are less than 1mm long
    // (stl.triangles.push(...) and stl.header.num_triangles++)

    // find out the range of X coordinates
    for t in &stl.triangles {
        MINX = f32min(MINX, t.v1[0]);
        MINX = f32min(MINX, t.v2[0]);
        MINX = f32min(MINX, t.v3[0]);

        MAXX = f32max(MAXX, t.v1[0]);
        MAXX = f32max(MAXX, t.v2[0]);
        MAXX = f32max(MAXX, t.v3[0]);
    }

    println!("X ranges from {} to {}", MINX, MAXX);

    let mut newtris = Vec::new();

    // rewrite vertices to wrap them around a cylinder
    for t in &stl.triangles {
        newtris.push(stl::Triangle {
            normal: t.normal, // XXX: incorrect, but we don't care
            v1: wrapvertex(t.v1),
            v2: wrapvertex(t.v2),
            v3: wrapvertex(t.v3),
            attr_byte_count: t.attr_byte_count,
        });
    }

    let newfile = stl::BinaryStlFile {
        header: stl.header,
        triangles: newtris,
    };

    let mut out = File::create(stlfilename.to_owned() + ".wrap").unwrap();

    // write out stl file
    assert!(stl::write_stl(&mut out, &newfile).is_ok());
} }
