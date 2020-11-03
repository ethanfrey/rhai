#![cfg(not(any(feature = "no_index", feature = "no_module")))]

use rhai::module_resolvers::StaticModuleResolver;
use rhai::plugin::*;
use rhai::{Engine, EvalAltResult, INT};

mod test {
    use rhai::plugin::*;

    #[export_module]
    pub mod special_array_package {
        use rhai::{Array, INT};

        pub const MYSTIC_NUMBER: INT = 42;

        #[cfg(not(feature = "no_object"))]
        pub mod feature {
            use rhai::{Array, Dynamic, EvalAltResult};

            #[rhai_fn(get = "foo", return_raw)]
            #[inline(always)]
            pub fn foo(array: &mut Array) -> Result<Dynamic, Box<EvalAltResult>> {
                Ok(array[0].clone())
            }
        }

        pub fn hash(_text: String) -> INT {
            42
        }
        pub fn hash2(_text: &str) -> INT {
            42
        }

        #[rhai_fn(name = "test", name = "hi")]
        #[inline(always)]
        pub fn len(array: &mut Array, mul: INT) -> INT {
            (array.len() as INT) * mul
        }
        #[rhai_fn(name = "+")]
        #[inline(always)]
        pub fn funky_add(x: INT, y: INT) -> INT {
            x / 2 + y * 2
        }
    }
}

macro_rules! gen_unary_functions {
    ($op_name:ident = $op_fn:ident ( $($arg_type:ident),+ ) -> $return_type:ident) => {
        mod $op_name { $(
            #[allow(non_snake_case)]
            pub mod $arg_type {
                use super::super::*;

                #[export_fn(name="test")]
                pub fn single(x: $arg_type) -> $return_type {
                    super::super::$op_fn(x)
                }
            }
        )* }
    }
}

macro_rules! reg_functions {
    ($mod_name:ident += $op_name:ident :: $func:ident ( $($arg_type:ident),+ )) => {
        $(register_exported_fn!($mod_name, stringify!($op_name), $op_name::$arg_type::$func);)*
    }
}

fn make_greeting(n: impl std::fmt::Display) -> String {
    format!("{} kitties", n)
}

gen_unary_functions!(greet = make_greeting(INT, bool, char) -> String);

#[test]
fn test_plugins_package() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    let mut m = Module::new();
    combine_with_exported_module!(&mut m, "test", test::special_array_package);
    engine.load_package(m);

    reg_functions!(engine += greet::single(INT, bool, char));

    #[cfg(not(feature = "no_object"))]
    assert_eq!(engine.eval::<INT>("let a = [1, 2, 3]; a.foo")?, 1);

    assert_eq!(engine.eval::<INT>(r#"hash("hello")"#)?, 42);
    assert_eq!(engine.eval::<INT>(r#"hash2("hello")"#)?, 42);
    assert_eq!(engine.eval::<INT>("let a = [1, 2, 3]; test(a, 2)")?, 6);
    assert_eq!(engine.eval::<INT>("let a = [1, 2, 3]; hi(a, 2)")?, 6);
    assert_eq!(engine.eval::<INT>("let a = [1, 2, 3]; test(a, 2)")?, 6);
    assert_eq!(engine.eval::<INT>("2 + 2")?, 5);
    assert_eq!(
        engine.eval::<String>("let a = [1, 2, 3]; greet(test(a, 2))")?,
        "6 kitties"
    );

    let mut resolver = StaticModuleResolver::new();
    resolver.insert("test", exported_module!(test::special_array_package));

    engine.set_module_resolver(Some(resolver));
    assert_eq!(
        engine.eval::<INT>(r#"import "test" as test; test::MYSTIC_NUMBER"#)?,
        42
    );

    Ok(())
}
