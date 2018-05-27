// TODO: add debug!() logs everywhere

use std::collections::{hash_map, HashMap};
use std::default::Default;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct ComponentContainer<Id: Hash + Eq, V> {
    data: HashMap<Id, V>,
}

impl<Id: Hash + Eq + Copy, V: Clone> Default for ComponentContainer<Id, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Id: Hash + Eq + Copy, V: Clone> ComponentContainer<Id, V> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get_opt(&self, id: Id) -> Option<&V> {
        self.data.get(&id)
    }

    pub fn get(&self, id: Id) -> &V {
        self.get_opt(id).unwrap()
    }

    pub fn get_opt_mut(&mut self, id: Id) -> Option<&mut V> {
        self.data.get_mut(&id)
    }

    pub fn get_mut(&mut self, id: Id) -> &mut V {
        self.get_opt_mut(id).unwrap()
    }

    pub fn insert(&mut self, id: Id, data: V) {
        assert!(self.get_opt(id).is_none());
        self.data.insert(id, data);
    }

    pub fn remove(&mut self, id: Id) {
        assert!(self.get_opt(id).is_some());
        self.data.remove(&id);
    }

    pub fn ids(&self) -> IdIter<Id, V> {
        IdIter::new(&self.data)
    }
}

#[derive(Clone, Debug)]
pub struct IdIter<'a, Id: 'a, V: 'a> {
    iter: hash_map::Iter<'a, Id, V>,
}

impl<'a, Id: Eq + Hash + Clone + 'a, V: 'a> IdIter<'a, Id, V> {
    pub fn new(map: &'a HashMap<Id, V>) -> Self {
        Self { iter: map.iter() }
    }
}

impl<'a, Id: Copy + 'a, V> Iterator for IdIter<'a, Id, V> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((&id, _)) = self.iter.next() {
            Some(id)
        } else {
            None
        }
    }
}

#[macro_export]
macro_rules! zcomponents_storage {
    ($struct_name:ident<$id_type:ty>: { $($component:ident: $t:ty,)* } ) => {
        use std::collections::HashMap;

        #[derive(Clone, Debug)]
        pub struct $struct_name {
            $(
                pub $component: $crate::ComponentContainer<$id_type, $t>,
            )*
            next_obj_id: $id_type,
            ids: HashMap<$id_type, ()>,
        }

        #[allow(dead_code)]
        impl $struct_name {
            pub fn new() -> Self {
                debug!("$struct_name: new"); // TODO: check if it works as expected
                Self {
                    $(
                        $component: $crate::ComponentContainer::new(),
                    )*
                    next_obj_id: Default::default(),
                    ids: HashMap::new(),
                }
            }

            pub fn alloc_id(&mut self) -> $id_type {
                let id = self.next_obj_id;
                self.next_obj_id.0 += 1;
                self.ids.insert(id, ());
                debug!("$struct_name: alloc_id {:?}", id);
                id
            }

            pub fn ids(&self) -> $crate::IdIter<$id_type, ()> {
                $crate::IdIter::new(&self.ids)
            }

            pub fn is_exist(&self, id: $id_type) -> bool {
                $(
                    if self.$component.get_opt(id).is_some() {
                        debug!("is_exist {:?}: true", id);
                        return true;
                    }
                )*
                debug!("is_exist {:?}: false", id);
                false
            }

            pub fn remove(&mut self, id: $id_type) {
                debug!("remove {:?}", id);
                $(
                    if self.$component.get_opt(id).is_some() {
                        self.$component.remove(id);
                    }
                )*
            }

            pub fn debug_string(&self, id: $id_type) -> String {
                let mut s = String::new();
                $(
                    if let Some(component) = self.$component.get_opt(id) {
                        s.push_str(&format!("{:?} ", component));
                    }
                )*
                s
            }
        }
    }
}
