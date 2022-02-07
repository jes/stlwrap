use std::fs::File;
use stl_io;
use stl_io::{Normal,Vertex};
use clap::{Arg, App};

// TODO: should be a bit object-oriented or something, instead of globals
static mut MINX:f32 = std::f32::MAX;
static mut MAXX:f32 = std::f32::MIN;
static mut MAXLENGTH:f32 = 3.0;

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
    let k = (x - MINX) / (MAXX - MINX + 0.01); // ranges from 0 to 1
    return k * std::f32::consts::PI * 2.0;
}

// wrap (x,y) from the flat model onto the cylindrical model
unsafe fn wrapxy(x: f32, y: f32) -> (f32, f32) {
    let angle = x2angle(x);
    let radius = y;
    return (radius * angle.cos(), radius * angle.sin());
}

unsafe fn wrapvertex(v: Vertex) -> Vertex {
    let (x,y) = wrapxy(v[0], v[1]);
    return Vertex::new([x,y,v[2]]);
}

fn sidelength(v1: Vertex, v2: Vertex) -> f32 {
    let dx = v1[0] - v2[0];
    let dy = v1[1] - v2[1];
    let dz = v1[2] - v2[2];
    let l = (dx*dx+dy*dy+dz*dz).sqrt();
    return l;
}

unsafe fn sidestoolong(t: &stl_io::Triangle) -> bool {
    return sidelength(t.vertices[0], t.vertices[1]) > MAXLENGTH
        || sidelength(t.vertices[1], t.vertices[2]) > MAXLENGTH
        || sidelength(t.vertices[2], t.vertices[0]) > MAXLENGTH;
}

fn midpoint(v1: Vertex, v2: Vertex) -> Vertex {
    return Vertex::new([
        (v1[0]+v2[0])/2.0,
        (v1[1]+v2[1])/2.0,
        (v1[2]+v2[2])/2.0,
    ]);
}

unsafe fn subdivide(t: stl_io::Triangle, triangles: &mut Vec<stl_io::Triangle>) {
    if !sidestoolong(&t) {
        triangles.push(t);
        return;
    }

    let v1 = t.vertices[0];
    let v2 = t.vertices[1];
    let v3 = t.vertices[2];

    let v12 = midpoint(v1, v2);
    let v23 = midpoint(v2, v3);
    let v31 = midpoint(v3, v1);

    let t1 = stl_io::Triangle {
        normal: t.normal,
        vertices: [v1, v12, v31],
    };
    let t2 = stl_io::Triangle {
        normal: t.normal,
        vertices: [v2,v23,v12],
    };
    let t3 = stl_io::Triangle {
        normal: t.normal,
        vertices: [v3,v31,v23],
    };
    let t4 = stl_io::Triangle {
        normal: t.normal,
        vertices:[v12,v23,v31],
    };

    subdivide(t1, triangles);
    subdivide(t2, triangles);
    subdivide(t3, triangles);
    subdivide(t4, triangles);
}

// https://math.stackexchange.com/a/305914
// return the normal vector of a triangle with the given 3 vertices
fn trinormal(v1: Vertex, v2: Vertex, v3: Vertex) -> Normal {
    let v = [v2[0]-v1[0], v2[1]-v1[1], v2[2]-v1[2]];
    let w = [v3[0]-v1[0], v3[1]-v1[1], v3[2]-v1[2]];

    let nx = (v[1]*w[2]) - (v[2]*w[1]);
    let ny = (v[2]*w[0]) - (v[0]*w[2]);
    let nz = (v[0]*w[1]) - (v[1]*w[0]);

    let len = (nx*nx+ny*ny+nz*nz).sqrt();
    return Normal::new([nx/len, ny/len, nz/len]);
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

    // read in the stl file
    let mut file = File::open(stlfilename).unwrap();
    let stl = stl_io::read_stl(&mut file).unwrap();

    // find out the range of X coordinates
    for t in &stl.faces {
        MINX = f32min(MINX, stl.vertices[t.vertices[0]][0]);
        MINX = f32min(MINX, stl.vertices[t.vertices[1]][0]);
        MINX = f32min(MINX, stl.vertices[t.vertices[2]][0]);

        MAXX = f32max(MAXX, stl.vertices[t.vertices[0]][0]);
        MAXX = f32max(MAXX, stl.vertices[t.vertices[1]][0]);
        MAXX = f32max(MAXX, stl.vertices[t.vertices[2]][0]);
    }

    let mut newtris = Vec::new();

    // subdivide triangles with long sides so that all sides are less than 1mm long
    for t in stl.faces {
        subdivide(stl_io::Triangle {
            normal: t.normal,
            vertices: [stl.vertices[t.vertices[0]],
                       stl.vertices[t.vertices[1]],
                       stl.vertices[t.vertices[2]]],
        }, &mut newtris);
    }

    let mut newtris2 = Vec::new();

    // rewrite vertices to wrap them around a cylinder
    for t in &newtris {
        let v1 = wrapvertex(t.vertices[0]);
        let v2 = wrapvertex(t.vertices[1]);
        let v3 = wrapvertex(t.vertices[2]);
        newtris2.push(stl_io::Triangle {
            normal: trinormal(v1,v2,v3),
            vertices: [v1, v2,v3],
        });
    }

    // write out stl file
    let mut out = File::create(stlfilename.to_owned() + ".wrap").unwrap();
    stl_io::write_stl(&mut out, newtris2.iter()).unwrap();
} }
