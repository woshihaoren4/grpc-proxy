use crate::app::DynMap;
use crate::infra::dynamic::DynClient;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct MapList {
    list: Vec<(String, String, Arc<DynClient>)>, //name,path,dyn client
}
pub struct DynMapDefault {
    map: Vec<RwLock<MapList>>,
    index: AtomicUsize,
}

impl MapList {
    #[allow(dead_code)]
    fn new(list: Vec<(String, String, Arc<DynClient>)>) -> Self {
        Self { list }
    }
    fn insert(
        list: &mut Vec<(String, String, Arc<DynClient>)>,
        name: String,
        path: String,
        dc: DynClient,
    ) {
        //短的在前，长的在后
        for (i, (n, _, _)) in list.iter().enumerate() {
            if n.eq(&name) {
                list[i] = (name, path, Arc::new(dc));
                return;
            }
        }
        let mut i = list.len() as isize - 1;
        while i >= 0 {
            if list[i as usize].0.len() < path.len() {
                list.insert((i as usize) + 1, (name, path, Arc::new(dc)));
                return;
            }
            i -= 1;
        }
        list.insert(0, (name, path, Arc::new(dc)));
    }
}

impl DynMapDefault {
    fn reset(&self, old_index: usize, list: Vec<(String, String, Arc<DynClient>)>) -> usize {
        let new_index = if old_index == 1 { 0 } else { 1 };
        let rw = self.map.get(new_index).unwrap();
        let mut rw = rw.write().unwrap();
        let rw_map = rw.deref_mut();
        rw_map.list = list;
        return new_index;
    }
}

impl Default for DynMapDefault {
    fn default() -> Self {
        let map = vec![
            RwLock::new(MapList::default()),
            RwLock::new(MapList::default()),
        ];
        let index = AtomicUsize::new(0);
        Self { map, index }
    }
}

impl DynMap for DynMapDefault {
    fn get(&self, path: String) -> Option<Arc<DynClient>> {
        let index = self.index.load(Ordering::Relaxed);
        let rw = self.map.get(index).unwrap();
        let binding = rw.read();
        let map_r = binding.as_ref().unwrap();
        for (_, p, client) in map_r.list.iter().rev() {
            if path.starts_with(p) {
                return Some(client.clone());
            }
        }
        return None;
    }

    //fixme 需要加一个写操作的互斥锁，否则极小概率导致死锁
    fn set(&self, name: String, path: String, dc: DynClient) {
        let index = self.index.load(Ordering::Relaxed);
        let rw = self.map.get(index).unwrap();
        let binding = rw.read();
        let map_r = binding.as_ref().unwrap();
        let mut map = map_r.list.clone();
        drop(rw);
        MapList::insert(&mut map, name, path, dc);
        let new_index = self.reset(index, map);
        self.index.store(new_index, Ordering::Relaxed);
    }
}
