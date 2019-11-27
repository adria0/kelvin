/// Model test suite for maps
///
/// Usage example: `quickcheck_map!(|| HAMT::new());`
#[macro_export]
macro_rules! quickcheck_map {
    ($new_map:expr) => {
        // mod inner_mod {
        use tempfile::tempdir;

        #[allow(unused)]
        use std::collections::HashMap;

        #[allow(unused)]
        use crate::{KeyValIterable, LeafIterable, Store};
        use quickcheck::{quickcheck, Arbitrary, Gen};

        use rand::Rng;

        const KEY_SPACE: u8 = 20;

        #[derive(Clone, Debug)]
        pub enum Op {
            Insert(u8, u8),
            Get(u8),
            GetMut(u8),
            Remove(u8),
            RemoveAll,
            Iter,
            IterMut,
            Values,
            ValuesMut,
            Keys,
            Persist,
            PersistRestore,
        }

        impl Arbitrary for Op {
            fn arbitrary<G: Gen>(g: &mut G) -> Op {
                let k: u8 = g.gen_range(0, KEY_SPACE);
                let op = g.gen_range(0, 12);
                match op {
                    0 => Op::Insert(k, g.gen()),
                    1 => Op::Iter,
                    2 => Op::IterMut,
                    3 => Op::Get(k),
                    4 => Op::GetMut(k),
                    5 => Op::Remove(k),
                    6 => Op::RemoveAll,
                    7 => Op::Values,
                    8 => Op::ValuesMut,
                    9 => Op::Keys,
                    10 => Op::Persist,
                    11 => Op::PersistRestore,
                    _ => unreachable!(),
                }
            }
        }

        fn run_ops(ops: Vec<Op>) -> bool {
            let dir = tempdir().unwrap();
            let store = Store::<Blake2b>::new(&dir.path()).unwrap();

            let mut test_a = $new_map();

            let mut model = HashMap::new();

            for op in ops {
                match op {
                    Op::Insert(k, v) => {
                        let a = test_a.insert(k, v).unwrap();
                        let b = model.insert(k, v);
                        assert_eq!(a, b);
                    }

                    Op::Iter => {
                        let mut a: Vec<_> = test_a
                            .iter()
                            .map(|res| res.unwrap())
                            .cloned()
                            .collect();
                        let mut b: Vec<_> = model
                            .iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect();

                        a.sort();
                        b.sort();

                        assert_eq!(a, b);
                    }

                    Op::IterMut => {
                        for (_, value) in test_a.iter_mut().map(|r| r.unwrap())
                        {
                            *value = value.wrapping_add(1)
                        }

                        for (_, value) in model.iter_mut() {
                            *value = value.wrapping_add(1)
                        }
                    }

                    Op::Get(k) => {
                        let a = test_a.get(&k).unwrap();
                        let b = model.get(&k);

                        dbg!(a.is_some(), b.is_some());

                        match (a, b) {
                            (Some(a), Some(b)) => {
                                assert!(*a == *b);
                            }
                            (None, None) => (),
                            (Some(_), None) => panic!("test has kv, model not"),
                            (None, Some(_)) => panic!("model has kv, test not"),
                        };
                    }

                    Op::GetMut(k) => {
                        let a = test_a
                            .get_mut(&k)
                            .unwrap()
                            .map(|mut val| *val = val.wrapping_add(1));
                        let b = model
                            .get_mut(&k)
                            .map(|val| *val = val.wrapping_add(1));

                        assert!(a == b)
                    }

                    Op::Remove(k) => {
                        let a = test_a.remove(&k).unwrap();
                        let c = model.remove(&k);

                        assert!(a == c);
                    }

                    Op::RemoveAll => {
                        model.clear();
                        let mut keys = vec![];
                        for (key, _) in test_a.iter().map(|res| res.unwrap()) {
                            keys.push(key.clone());
                        }
                        for key in keys {
                            test_a.remove(&key).unwrap();
                        }
                        test_a.assert_correct_empty_state();
                    }

                    Op::Values => {
                        let mut a: Vec<_> =
                            test_a.values().map(|v| *v.unwrap()).collect();

                        let mut c: Vec<_> =
                            model.values().map(|v| *v).collect();

                        a.sort();
                        c.sort();

                        assert!(a == c);
                    }

                    Op::ValuesMut => {
                        let _res = test_a
                            .values_mut()
                            .map(|v| {
                                let v = v.unwrap();
                                *v = v.wrapping_add(1);
                                ()
                            })
                            .collect::<Vec<_>>();

                        let _res = model
                            .values_mut()
                            .map(|v| *v = v.wrapping_add(1))
                            .collect::<Vec<_>>();

                        let mut a: Vec<_> =
                            test_a.values().map(|v| *v.unwrap()).collect();

                        let mut c: Vec<_> =
                            model.values().map(|v| *v).collect();

                        a.sort();
                        c.sort();

                        assert!(a == c);
                    }
                    Op::Keys => {
                        let mut a: Vec<_> =
                            test_a.keys().map(|v| *v.unwrap()).collect();

                        let mut c: Vec<_> = model.keys().map(|k| *k).collect();

                        a.sort();
                        c.sort();

                        assert!(a == c);
                    }
                    Op::Persist => {
                        store.persist(&mut test_a).unwrap();
                    }
                    Op::PersistRestore => {
                        let snapshot = store.persist(&mut test_a).unwrap();
                        test_a = store.restore(&snapshot).unwrap();
                    }
                };
            }
            true
        }

        quickcheck! {
            fn map(ops: Vec<Op>) -> bool {
                run_ops(ops)
            }
        }

        use Op::*;

        // regressions
        #[test]
        fn regression_pre_persist_fail() {
            assert!(run_ops(vec![Insert(6, 241), Insert(9, 147), Persist]))
        }
    };
}
