use criterion::{criterion_group, criterion_main, Criterion};
use gravsim_simulation::tree::Node;
use gravsim_simulation::{MassData, MassDistribution, Simulation, Star};
use nalgebra::Vector2;
use once_cell::sync::OnceCell;

static OBJS_1K: OnceCell<Vec<MassData>> = OnceCell::new();
static OBJS_5K: OnceCell<Vec<MassData>> = OnceCell::new();

fn build_tree(c: &mut Criterion) {
    let objs_1k = OBJS_1K
        .get_or_try_init(|| bincode::deserialize(include_bytes!("test_data/stars_1k.bin")))
        .unwrap();

    let objs_5k = OBJS_5K
        .get_or_try_init(|| bincode::deserialize(include_bytes!("test_data/stars_5k.bin")))
        .unwrap();

    c.bench_function("build-tree 1k", |b| {
        b.iter(|| {
            let mut tree = Node::new_root(Vector2::repeat(-500.0), 1000.0);

            for mass_data in objs_1k {
                tree.insert(mass_data);
            }
            tree
        })
    });
    c.bench_function("build-tree 5k", |b| {
        b.iter(|| {
            let mut tree = Node::new_root(Vector2::repeat(-500.0), 1000.0);

            for mass_data in objs_5k {
                tree.insert(mass_data);
            }
            tree
        })
    });
}

fn update_simulation(c: &mut Criterion) {
    let objs_1k = OBJS_1K
        .get_or_try_init(|| bincode::deserialize(include_bytes!("test_data/stars_1k.bin")))
        .unwrap();

    let objs_5k = OBJS_5K
        .get_or_try_init(|| bincode::deserialize(include_bytes!("test_data/stars_5k.bin")))
        .unwrap();

    let mut simulation = Simulation::new(
        objs_1k
            .iter()
            .map(|obj| Star::new(obj.position, Vector2::zeros(), obj.mass)),
        MassDistribution::new(1.0, 1.0),
    );
    c.bench_function("step 1k", |b| b.iter(|| simulation.update()));

    let mut simulation = Simulation::new(
        objs_5k
            .iter()
            .map(|obj| Star::new(obj.position, Vector2::zeros(), obj.mass)),
        MassDistribution::new(1.0, 1.0),
    );
    c.bench_function("step 5k", |b| b.iter(|| simulation.update()));
}

criterion_group!(gravity, update_simulation);
criterion_main!(gravity);

// #[test]
pub fn gen_test_data() {
    use rand::{Rng, SeedableRng};
    use rand_xorshift::XorShiftRng;
    use std::path::Path;

    let mut rng = XorShiftRng::from_entropy();

    for n in [1000, 5000] {
        let mass_datas: Vec<_> = (0..n)
            .map(|_| MassData {
                position: Vector2::from_fn(|_, _| rng.gen::<f32>() * 1000.0 - 500.0),
                mass: 1.0,
            })
            .collect();

        let bytes = bincode::serialize(&mass_datas).unwrap();
        std::fs::write(
            Path::new(&format!("benches/test_data/stars_{}k.bin", n / 1000)),
            bytes,
        )
        .unwrap();
    }
}

// #[test]
pub fn test_test_data() {
    use std::path::Path;

    for n in [1000, 5000] {
        let stars: Vec<MassData> = bincode::deserialize(
            &std::fs::read(Path::new(&format!(
                "benches/test_data/stars_{}k.bin",
                n / 1000
            )))
            .unwrap(),
        )
        .unwrap();

        assert_eq!(stars.len(), n as usize);
    }
}
