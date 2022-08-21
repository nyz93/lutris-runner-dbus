use std::any::Any;

use anyhow::Result;

use crate::library::{Library, LocalLibrary};

fn dbus_err_string<T: std::fmt::Debug>(e: T) -> dbus::MethodErr {
    dbus::MethodErr::failed(&format!("{:?}", e))
}

fn create_server<L: Library + Send + Any + 'static>(l: L) -> Result<dbus_crossroads::Crossroads> {
    let mut cr = dbus_crossroads::Crossroads::new();
    let token = cr.register("org.kde.krunner1", |b| {
        b.method("Teardown", (), (), |_, s: &mut L, ()| Ok(s.teardown()));
        b.method("Actions", (), ("matches",), |_, _: &mut L, ()| {
            Ok((vec![
                ("open-dir", "Open Directory", "folder-open-symbolic").to_owned()
            ],))
        })
        .annotate("org.qtproject.QtDBus.QtTypeName.Out0", "RemoteActions");
        b.method(
            "Run",
            ("matchId", "actionId"),
            (),
            |_, l: &mut L, (match_id, action): (_, String)| match action.as_str() {
                "open-dir" => l.open_dir(match_id).map_err(dbus_err_string),
                &_ => l.run_game(match_id).map_err(dbus_err_string),
            },
        );
        b.method(
            "Match",
            ("query",),
            ("matches",),
            |_, t: &mut L, (query,)| {
                t.query_game(&dbg!(query))
                    .map(|games| (games,))
                    .map_err(dbus_err_string)
            },
        )
        .annotate("org.qtproject.QtDBus.QtTypeName.Out0", "RemoteMatches");
    });
    cr.insert("/KRunnerLutris", &[token], l);
    Ok(cr)
}
#[cfg(test)]
#[allow(unused)]
mod tests {
    use crate::dbus::create_server;
    use crate::library::{Game, Library};
    use dbus::arg::{RefArg, Variant};
    use std::cell::RefCell;

    struct TestLibrary {}
    impl TestLibrary {
        fn new() -> Self {
            TestLibrary {}
        }
    }

    impl Library for TestLibrary {
        fn query_game(&mut self, query: &String) -> anyhow::Result<Vec<Game>> {
            if query == "test" {
                Ok(vec![Game {
                    category: "cat".to_string(),
                    name: "name".to_string(),
                    src: "src".to_string(),
                    id: 10,
                    directory: Some("a".to_string()),
                    icon: "icon".to_string(),
                }])
            } else {
                Ok(vec![])
            }
        }

        fn run_game(&self, id: String) -> anyhow::Result<()> {
            todo!()
        }

        fn open_dir(&self, id: String) -> anyhow::Result<()> {
            todo!()
        }

        fn teardown(&mut self) {
            todo!()
        }
    }

    #[test]
    fn query() {
        let mut s = create_server(TestLibrary::new()).unwrap();
        struct A {
            replies: std::cell::RefCell<Vec<dbus::Message>>,
        }
        impl dbus::channel::Sender for A {
            fn send(&self, reply: dbus::Message) -> Result<u32, ()> {
                self.replies.borrow_mut().push(reply);
                Ok(1)
            }
        }
        let conn = A {
            replies: RefCell::new(vec![]),
        };
        let mut msg = dbus::Message::new_method_call(
            ":session",
            "/KRunnerLutris",
            "org.kde.krunner1",
            "Match",
        )
        .unwrap()
        .append1("test");
        msg.set_serial(19);
        s.handle_message(msg, &conn).unwrap();
        let (id, name, icon, n, fit, props) = &conn.replies.borrow()[0]
            .get1::<Vec<(String, String, String, i32, f64, dbus::arg::PropMap)>>()
            .unwrap()[0];
        assert_eq!(id, "src_10");
        assert_eq!(name, "name");
        assert_eq!(icon, "icon");
        assert_eq!(*n, 30);
        assert_eq!(*fit, 100.0);
        let actions: Vec<_> = props
            .get("actions")
            .unwrap()
            .as_iter()
            .unwrap()
            .next()
            .unwrap()
            .as_iter()
            .unwrap()
            .map(|x| x.as_str().unwrap())
            .collect();
        assert_eq!(actions, vec!["open-dir"]);
        assert_eq!(props.get("category").unwrap().as_str(), Some("cat"));
    }
}

pub fn serve() -> Result<()> {
    let cr = create_server(LocalLibrary::new()?)?;
    let conn = dbus::blocking::Connection::new_session()?;
    conn.request_name("org.kde.KRunnerLutris", true, true, true)?;
    cr.serve(&conn)?;
    Ok(())
}
