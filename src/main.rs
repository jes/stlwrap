use std::fs::File;
use clap::{Arg, App};

// TODO: should be a bit object-oriented or something, instead of globals
static mut MINX:f32 = std::f32::MAX;
static mut MAXX:f32 = std::f32::MIN;
static mut MAXLENGTH:f32 = 3.0;
static mut MINLENGTH:f32 = 0.001;

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

fn sidelength(v1: [f32;3], v2: [f32;3]) -> f32 {
    let dx = v1[0] - v2[0];
    let dy = v1[1] - v2[1];
    let dz = v1[2] - v2[2];
    let l = (dx*dx+dy*dy+dz*dz).sqrt();
    println!("Side length {},{},{} = {}", dx,dy,dz,l);
    return l;
}

unsafe fn sidestoolong(t: &stl::Triangle) -> bool {
    return sidelength(t.v1, t.v2) > MAXLENGTH
        || sidelength(t.v2, t.v3) > MAXLENGTH
        || sidelength(t.v3, t.v1) > MAXLENGTH;
}

unsafe fn subdivide(t: stl::Triangle, triangles: &mut Vec<stl::Triangle>) {
    if !sidestoolong(&t) {
        triangles.push(t);
        return;
    }

    // split the triangle into 3 smaller ones, each one using 2 of the existing vertices
    // plus a new vertex at the centroid of the triangle, and recurse
    let centroid = [
        (t.v1[0]+t.v2[0]+t.v3[0])/3.0,
        (t.v1[1]+t.v2[1]+t.v3[1])/3.0,
        (t.v1[2]+t.v2[2]+t.v3[2])/3.0,
    ];

    let t1 = stl::Triangle {
        normal: t.normal, // XXX: incorrect, but we don't care
        v1: t.v1,
        v2: t.v2,
        v3: centroid,
        attr_byte_count: t.attr_byte_count,
    };
    let t2 = stl::Triangle {
        normal: t.normal, // XXX: incorrect, but we don't care
        v1: t.v1,
        v2: centroid,
        v3: t.v3,
        attr_byte_count: t.attr_byte_count,
    };
    let t3 = stl::Triangle {
        normal: t.normal, // XXX: incorrect, but we don't care
        v1: centroid,
        v2: t.v2,
        v3: t.v3,
        attr_byte_count: t.attr_byte_count,
    };

    subdivide(t1, triangles);
    subdivide(t2, triangles);
    subdivide(t3, triangles);
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
    MAXLENGTH = maxlength;

    println!("Max. side length is {}", maxlength);

    // read in the stl file
    let mut file = File::open(stlfilename).unwrap();
    let stl = stl::read_stl(&mut file).unwrap();

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

    // subdivide triangles with long sides so that all sides are less than 1mm long
    for t in stl.triangles {
        subdivide(t, &mut newtris);
    }

    let mut newtris2 = Vec::new();

    // rewrite vertices to wrap them around a cylinder
    for t in &newtris {
        newtris2.push(stl::Triangle {
            normal: t.normal, // XXX: incorrect, but we don't care
            v1: wrapvertex(t.v1),
            v2: wrapvertex(t.v2),
            v3: wrapvertex(t.v3),
            attr_byte_count: t.attr_byte_count,
        });
    }

    // create new stl file object with new triangles
    let newfile = stl::BinaryStlFile {
        header: stl::BinaryStlHeader {
            header: stl.header.header,
            num_triangles: newtris.len() as u32,
        },
        triangles: newtris2,
    };

    // write out stl file
    let mut out = File::create(stlfilename.to_owned() + ".wrap").unwrap();
    assert!(stl::write_stl(&mut out, &newfile).is_ok());
} }
