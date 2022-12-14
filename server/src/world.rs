use crate::vanilla;
use glam::{IVec2, DVec2, UVec2};
use prost::Message;
use std::collections::HashMap;

pub trait EncodeNet {
    type NetStruct: prost::Message;
    fn encode_net(&self) -> Self::NetStruct;
    fn encode_ws(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let net_struct = self.encode_net();
        buf.reserve(net_struct.encoded_len());
        net_struct.encode(&mut buf).unwrap();

        buf
    }
}

#[derive(Debug, Clone)]
pub struct World {
    pub chunks: HashMap<IVec2, Chunk>,
    pub entities: HashMap<u32, Entity>,
    pub background: Option<[u8; 3]>,

    pub next_entity_id: u32,
}

impl World {
    pub fn new(chunks: Vec<Chunk>, background: Option<[u8; 3]>) -> Self {
        let mut world = Self {
            chunks: HashMap::new(),
            entities: HashMap::new(),
            background,

            next_entity_id: 0,
        };

        for chunk in chunks {
            world.chunks.insert(chunk.pos, chunk);
        }

        world
    }

    pub fn spawn_entity(&mut self, pos: DVec2, size: UVec2) -> u32 {
        let entity = Entity { id: self.next_entity_id, pos, size };
        self.entities.insert(self.next_entity_id, entity);

        self.next_entity_id += 1;
        // uhmmmm
        self.next_entity_id - 1
    }
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: u32,
    pub pos: DVec2,
    pub size: UVec2,
}

impl EncodeNet for Entity {
    type NetStruct = vanilla::Entity;
    fn encode_net(&self) -> Self::NetStruct {
        let entity_net = vanilla::Entity {
            id: self.id,
            x: self.pos.x,
            y: self.pos.y,
            width: self.size.x,
            height: self.size.y,
        };

        entity_net
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub pos: IVec2,
    pub background: Option<[u8; 3]>,
    pub pixels: [[Option<Pixel>; 32]; 32],
    // For caching. expected to be correct
    pub pallete: Vec<[u8; 3]>,
    pub colors: [[u8; 32]; 32],
}

impl Chunk {
    pub fn new(pos: IVec2, pixels: Vec<Pixel>, background: Option<[u8; 3]>) -> Self {
        let mut chunk = Chunk {
            pos,
            background,
            pixels: [[None; 32]; 32],
            pallete: Vec::new(),
            colors: [[0; 32]; 32],
        };

        for pixel in pixels {
            let mut absoulte_coord_pixel = pixel.clone();
            absoulte_coord_pixel.pos += chunk.pos * 32;
            chunk.pixels[pixel.pos.y as usize][pixel.pos.x as usize] = Some(absoulte_coord_pixel);
        }

        chunk.rebuild_cache();
        chunk
    }

    fn rebuild_cache(&mut self) {
        // Maybe clear the pallete just in case?
        let colors = self.colors.flatten_mut();
        for (i, p) in self.pixels.flatten().iter().enumerate() {
            if let Some(pixel) = p {
                if let Some(pallete_index) = self.pallete.iter().position(|&i| i == pixel.color) {
                    colors[i] = pallete_index as u8 + 1;
                } else {
                    assert!(self.pallete.len() <= 255);
                    self.pallete.push(pixel.color);
                    colors[i] = self.pallete.len() as u8;
                }
            } else {
                colors[i] = 0;
            }
        }
    }

    fn encode_net_rebuild(&mut self) -> <Self as EncodeNet>::NetStruct {
        self.rebuild_cache();
        self.encode_net()
    }

    fn encode_ws_rebuild(&mut self) -> Vec<u8> {
        let mut buf = Vec::new();
        let net_struct = self.encode_net_rebuild();
        buf.reserve(net_struct.encoded_len());
        net_struct.encode(&mut buf).unwrap();

        buf
    }

}



impl EncodeNet for Chunk {
    type NetStruct = vanilla::Chunk;
    fn encode_net(&self) -> Self::NetStruct {
        let mut chunk_net = vanilla::Chunk {
            x: self.pos.x,
            y: self.pos.y,
            background: self.background.map(|a| a.to_vec()),
            pixels: Vec::new(),
            pallete: Vec::new(),
        };
        chunk_net.pallete = self.pallete.flatten().to_vec();
        chunk_net.pixels = self.colors.flatten().to_vec();

        chunk_net
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    color: [u8; 3],
    pos: IVec2,
    solid: bool,
}

impl Pixel {
    pub fn new(pos: IVec2, color: [u8; 3], solid: bool) -> Self {
        Self { color, pos, solid }
    }
}
