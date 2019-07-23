let dbg_impl = quote! {

        type Cb = std::boxed::Box<(dyn std::ops::Fn() + 'static)>;
        struct Vars {
            repr: Cb,
        }
        impl Vars {
            fn new(f: impl Fn()) -> Self {
                let cb = Box::new(f as Fn());
                Vars {
                    repr: cb,
                }
            }
        }

        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
        pub struct DebugCollect {
            pub args: std::collections::HashMap<String, String>,
        }

        impl DebugCollect {
           pub fn new(s: &str) -> Self {
               let d: DebugCollect = serde_json::from_str(s).unwrap();
               d
           }

            pub fn step(&self) -> std::io::Result<String> {
                println!("type var name or tab to auto-complete");
                fn print_loop(dbg: &DebugCollect) -> std::io::Result<String> {
                    let mut input = crossterm::input();
                    let line = input.read_line()?;
                    if let Some(var) = dbg.args.get(line.as_str()) {
                        Ok(var.clone())
                    } else {
                        println!("could not find variable '{}' in scope", line);
                        print_loop(&dbg)
                    }
                }
                print_loop(&self)
            }

            fn capature(&mut self, map: Vars) {

            }
        }
    };
