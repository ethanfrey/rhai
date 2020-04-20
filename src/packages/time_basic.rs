use super::logic::{eq, gt, gte, lt, lte, ne};
use super::math_basic::MAX_INT;
use super::{
    create_new_package, reg_binary, reg_none, reg_unary, Package, PackageLibrary,
    PackageLibraryStore,
};

use crate::fn_register::{map_dynamic as map, map_result as result};
use crate::parser::INT;
use crate::result::EvalAltResult;
use crate::token::Position;

use crate::stdlib::{ops::Deref, time::Instant};

pub struct BasicTimePackage(PackageLibrary);

impl Deref for BasicTimePackage {
    type Target = PackageLibrary;

    fn deref(&self) -> &PackageLibrary {
        &self.0
    }
}

impl Package for BasicTimePackage {
    fn new() -> Self {
        let mut pkg = create_new_package();
        Self::init(&mut pkg);
        Self(pkg.into())
    }

    fn get(&self) -> PackageLibrary {
        self.0.clone()
    }

    fn init(lib: &mut PackageLibraryStore) {
        #[cfg(not(feature = "no_std"))]
        {
            // Register date/time functions
            reg_none(lib, "timestamp", || Instant::now(), map);

            reg_binary(
                lib,
                "-",
                |ts1: Instant, ts2: Instant| {
                    if ts2 > ts1 {
                        #[cfg(not(feature = "no_float"))]
                        return Ok(-(ts2 - ts1).as_secs_f64());

                        #[cfg(feature = "no_float")]
                        {
                            let seconds = (ts2 - ts1).as_secs();

                            #[cfg(not(feature = "unchecked"))]
                            {
                                if seconds > (MAX_INT as u64) {
                                    return Err(EvalAltResult::ErrorArithmetic(
                                        format!(
                                            "Integer overflow for timestamp duration: {}",
                                            -(seconds as i64)
                                        ),
                                        Position::none(),
                                    ));
                                }
                            }
                            return Ok(-(seconds as INT));
                        }
                    } else {
                        #[cfg(not(feature = "no_float"))]
                        return Ok((ts1 - ts2).as_secs_f64());

                        #[cfg(feature = "no_float")]
                        {
                            let seconds = (ts1 - ts2).as_secs();

                            #[cfg(not(feature = "unchecked"))]
                            {
                                if seconds > (MAX_INT as u64) {
                                    return Err(EvalAltResult::ErrorArithmetic(
                                        format!(
                                            "Integer overflow for timestamp duration: {}",
                                            seconds
                                        ),
                                        Position::none(),
                                    ));
                                }
                            }
                            return Ok(seconds as INT);
                        }
                    }
                },
                result,
            );
        }

        reg_binary(lib, "<", lt::<Instant>, map);
        reg_binary(lib, "<=", lte::<Instant>, map);
        reg_binary(lib, ">", gt::<Instant>, map);
        reg_binary(lib, ">=", gte::<Instant>, map);
        reg_binary(lib, "==", eq::<Instant>, map);
        reg_binary(lib, "!=", ne::<Instant>, map);

        reg_unary(
            lib,
            "elapsed",
            |timestamp: Instant| {
                #[cfg(not(feature = "no_float"))]
                return Ok(timestamp.elapsed().as_secs_f64());

                #[cfg(feature = "no_float")]
                {
                    let seconds = timestamp.elapsed().as_secs();

                    #[cfg(not(feature = "unchecked"))]
                    {
                        if seconds > (MAX_INT as u64) {
                            return Err(EvalAltResult::ErrorArithmetic(
                                format!("Integer overflow for timestamp.elapsed(): {}", seconds),
                                Position::none(),
                            ));
                        }
                    }
                    return Ok(seconds as INT);
                }
            },
            result,
        );
    }
}
