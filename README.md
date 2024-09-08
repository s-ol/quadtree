# QuadTree

[![Current Crates.io Version](https://img.shields.io/crates/v/quadtree.svg)](https://crates.io/crates/quadtree)
[![Documentation](https://img.shields.io/badge/Docs-latest-blue)](https://docs.rs/quadtree/latest/quadtree/)

This QuadTree library provides efficient spatial querying capabilities for 2D points. It supports various operations such as insertion, and rectangular and circular querying, making it suitable for applications in areas such as gaming, geographical information systems, and real-time simulations.

## Features

- **Generic Implementation**: `QuadTree<T>` works with any data type `T` that implements the `Point` and `Clone` traits.
- **Spatial Queries**: Supports querying within spatial regions that implement the `Shape` trait (`Rect` and `Circle` are provided).
- **Dynamic Operations**: Efficiently perform mutating operations without full rebuilds.
    - insert
    - insert_many
    - delete
    - pop
- **Barnes-Hut Approximation**: Implement the `Mass` trait for your data type to use [Barnes-Hut approximation](https://en.wikipedia.org/wiki/Barnes%E2%80%93Hut_simulation) for n-body simulations. An interactive explanation of this algorithm can be found [here](https://jheer.github.io/barnes-hut/).
- **Serde Serialization**: Enable the `"serde"` feature to serialize the QuadTree and provided shapes. A `QuadTree<T>` will serialize into a sequence of items of type `T`.