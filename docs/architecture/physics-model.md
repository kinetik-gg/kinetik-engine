# Physics Model

Physics authoring is instance-based. Rapier-backed simulation is system-owned.

Core instances:

- `StaticBody3D`
- `RigidBody3D`
- `CharacterBody3D`
- `Area3D`
- `BoxCollider3D`
- `SphereCollider3D`
- `CapsuleCollider3D`
- `MeshCollider3D`

Ownership rule: a collider belongs to the nearest ancestor physics body. Collider without body is invalid at runtime; editor offers a quick fix.
