# `ZComponents` - a stupid component storage

I find "serious" ECS to be an overkill for turn-based game logic,
so I've created this simple library that does only one thing:
stores your components.

## Basic Example

```rust
#[macro_use]
extern crate zcomponents;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, Default)]
pub struct Id(i32);

#[derive(Clone, Debug)]
pub struct SomeComponent(pub i32);

#[derive(Clone, Debug)]
pub struct SomeFlag;

zcomponents_storage!(Storage<Id>: {
    component: SomeComponent,
    flag: SomeFlag,
});

let mut storage = Storage::new();

let id0 = storage.alloc_id();
storage.component.insert(id0, SomeComponent(0));

let id1 = storage.alloc_id();
storage.component.insert(id1, SomeComponent(1));
storage.flag.insert(id1, SomeFlag);

storage.component.get_mut(id0).0 += 1;

if let Some(component) = storage.component.get_opt_mut(id1) {
    component.0 += 1;
}

storage.flag.remove(id1);

storage.remove(id0);
```

See a more advanced example [in crate's documentation](src/lib.rs).

## Implementation

It's implemented as a simple macro and a bunch of naive `HashMap`s
so don't expect any outstanding performance.
