# TODO list

- [ ] Refactor hash map structures. Rust's native HashMap handle's collisions already, in the same way that we wrote the handler for the collisions.

    How Rust's HashMap Works
Hashing and Equality: The HashMap in Rust uses a hash function to compute a hash value for each key. This hash value determines where in the underlying array the value should be stored. Rust's HashMap uses a hashing algorithm called SipHash, which is a cryptographic hash function providing a good balance between speed and security.

Handling Collisions: When two different keys have the same hash value (a collision), Rust handles this by storing the colliding keys in a list (or bucket) at the same array index. It then checks the keys in the list using the PartialEq and Eq traits to find the correct one. If the key is already present, it updates the value; if not, it adds the new key-value pair.