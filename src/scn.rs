extern crate serde_json;
extern crate serde;
use std::fs;

use self::serde::Deserialize;

use crate::*;



pub fn generate_default() -> Scene {
    //let contents = fs::read_to_string(filename).unwrap();
    read_json(DEFAULT_JSON)
}

fn read_json(contents: &str) -> Scene {
    let scn: IpScene = serde_json::from_str(contents).unwrap();
    scn.to_scene()
}

impl IpScene {
    fn to_scene(self) -> Scene {
        let mut objects: Vec<Box<dyn SceneObject + Send + Sync>> = Vec::new();
        let mut lights = Vec::new();
        for object in self.objects {
            match object {
                IpObject::sphere(sphere) => {
                    objects.push(Box::new(Sphere::new(
                        sphere.center,
                        sphere.radius,
                        Material::new(sphere.color, sphere.reflective),
                    )));
                }
                IpObject::floor(floor) => {
                    objects.push(Box::new(Floor::new(
                        floor.y,
                        Material::new(floor.color, floor.reflective),
                    )));
                }
                IpObject::inline_obj(inline_obj) => {
                    let mut tris = read_obj(
                        inline_obj.contents.as_str(),
                        Material::new(inline_obj.color,inline_obj.reflective),
                    );
                    objects.append(&mut tris);
                }
                IpObject::extern_obj(extern_obj) => {
                    let mut tris = read_obj(
                        fs::read_to_string(extern_obj.filename).unwrap().as_str(),
                        Material::new(extern_obj.color,extern_obj.reflective),
                    );
                    objects.append(&mut tris);
                }
            }
        }
        for light in self.lights {
            match light {
                IpLight::point(pointlight) => {
                    lights.push(Light::Point(pointlight));
                }
                _=> {}
            }
        }
        let camera = Camera::new( 
            self.camera.origin,
            self.camera.direction,
            self.camera.focal_length
        );
        let world = World::new(
            self.background_color,
            1.0,
        );
        Scene::new(objects,lights,camera,world)
    }
}
#[derive(Deserialize)]
struct IpScene {
    objects: Vec<IpObject>,
    lights: Vec<IpLight>,
    camera: IpCamera,
    background_color: Color,
}
#[derive(Deserialize)]
enum IpObject {
    sphere(IpSphere),
    floor(IpFloor),
    inline_obj(ObjInline),
    extern_obj(ObjExtern),
}
#[derive(Deserialize)]
enum IpLight {
    point(PointLight),
}
#[derive(Deserialize)]
struct IpCamera {
    origin: Vec3,
    direction: Vec3,
    focal_length: f64,
}
#[derive(Deserialize)]
struct IpSphere {
    center: Vec3,
    radius: f64,
    reflective: bool,
    color: Color,
}
#[derive(Deserialize)]
struct IpFloor {
    y: f64,
    color: Color,
    reflective: bool,
}
#[derive(Deserialize)]
struct ObjInline {
    color: Color,
    reflective: bool,
    contents: String,
}
#[derive(Deserialize)]
struct ObjExtern {
    color: Color,
    reflective: bool,
    filename: String,
}


fn read_obj(contents: &str, material: Material) -> Vec<Box<dyn SceneObject + Send + Sync>> {
    let mut verts: Vec<Vec3> = Vec::new();
    let mut norms: Vec<Vec3> = Vec::new();
    let mut tris: Vec<Box<dyn SceneObject + Send + Sync>> = Vec::new();
    
    for line in contents.lines() {
        let is_vert = line.find("v ");
        if is_vert.is_some() { 
            let values: Vec<&str> = line.split(' ').collect();
            let x = values[1].parse::<f64>().unwrap();
            let y = values[2].parse::<f64>().unwrap();
            let z = values[3].parse::<f64>().unwrap();
            verts.push(Vec3::new(x,y,z));
        }
        let is_norm = line.find("vn ");
        if is_norm.is_some() { 
            let values: Vec<&str> = line.split(' ').collect();
            let x = values[1].parse::<f64>().unwrap();
            let y = values[2].parse::<f64>().unwrap();
            let z = values[3].parse::<f64>().unwrap();
            norms.push(Vec3::new(x,y,z));
        }
        
        let is_face = line.find("f ");
        if is_face.is_some() { 
            let values: Vec<&str> = line.split(' ').collect();
            let mut i = Vec::new();
            for value in &values[1..] {
                if value.is_empty() == false {
                    let ind: Vec<&str> = value.split('/').collect();
                    i.push( ind[0].parse::<usize>().unwrap()-1 );
                }
            }
            let mut n = Vec::new();
            for value in &values[1..] {
                if value.is_empty() == false {
                    let ind: Vec<&str> = value.split('/').collect();
                    n.push( ind[2].parse::<usize>().unwrap()-1 );
                }
            }
            tris.push( Box::new(Tri::new(
                verts[i[0]],verts[i[1]],verts[i[2]],
                norms[n[0]],norms[n[1]],norms[n[2]],
                material
            )));

            if i.len() > 3 { //quad
                tris.push( Box::new(Tri::new(
                    verts[i[0]],verts[i[2]],verts[i[3]],
                    norms[n[0]],norms[n[2]],norms[n[3]],
                    material
                )));
            }
        }
    }
    return tris;
}

const DEFAULT_JSON: &str = r#"
{
    "lights" : [
        {
            "point": {
                "origin": { "x": 1, "y": 7, "z": -3 },
                "strength": 1.5
            }
        }
    ],
    "camera" : {
        "origin": { "x": 0, "y": 2, "z": -6 },
        "direction": { "x": 0, "y": -0.1, "z": 1 },
        "focal_length": 1
    },
    "background_color": { "r": 0, "g": 0, "b": 255, "a": 255 },
    "objects" : [
        {
            "sphere": {
                "center": { "x": 0, "y": 1.3, "z": 0 },
                "radius": 1.3,
                "reflective": false,
                "color":  { "r": 255, "g": 0, "b": 0, "a": 255 }
            }
        },
        {
            "floor": {
                "y": 0,
                "reflective": false,
                "color": { "r": 100, "g": 50, "b": 100, "a": 255 }
            }
        }
    ]
}
"#;
