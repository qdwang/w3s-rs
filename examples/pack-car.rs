use std::{collections::HashMap, env, fs};

use w3s::writer::car_util::*;

fn main() {
    let args = env::args().collect::<Vec<_>>();

    match args.as_slice() {
        [_, path, name] => pack(path, name),
        _ => panic!("\n\nPlease input [folder_path_to_pack] and [output_car_name]\n\n"),
    }
}

fn files2blocks(
    dir_items: &[DirectoryItem],
    id_map: &mut HashMap<u64, Vec<UnixFsStruct>>,
    blocks_collector: &mut Vec<UnixFsStruct>,
) {
    for item in dir_items {
        match item {
            DirectoryItem::File(_, path, id) => {
                let data = fs::read(path).unwrap();
                let blocks = gen_blocks(data, 256 * 1024);
                id_map.insert(*id, blocks.clone());
                blocks_collector.extend(blocks);
            }
            DirectoryItem::Directory(name, sub_items) => {
                files2blocks(sub_items, id_map, blocks_collector);
            }
        }
    }
}

fn pack(path: &str, name: &str) {
    let (dir_items, _) = DirectoryItem::from_path(path, None).unwrap();

    let mut id_map: HashMap<u64, Vec<UnixFsStruct>> = HashMap::new();
    let mut file_blocks = vec![];
    files2blocks(&dir_items, &mut id_map, &mut file_blocks);

    let mut strcut_blocks = vec![];
    let root_blocks: Vec<_> = dir_items
        .iter()
        .map(|item| item.to_unixfs_struct(&id_map, &mut strcut_blocks))
        .collect();

    let mut total_blocks = vec![];
    total_blocks.extend(file_blocks);
    total_blocks.extend(strcut_blocks);

    for block in total_blocks.iter() {
        println!("{}", block);
    }

    let root = gen_dir(None, &root_blocks);
    println!("{}", root);

    let car = gen_car(&mut total_blocks, Some(root)).unwrap();

    fs::write(name, car).unwrap();

    println!("Done");
}
