use anyhow::Result;

use crate::lutris::{Game, Lutris};

impl Game {
    fn into_match(self: Game) -> (String, String, String, i32, f64, dbus::arg::PropMap) {
        use dbus::arg::{PropMap, RefArg, Variant};
        let mut props = PropMap::new();
        props.insert(
            "category".to_string(),
            Variant(format!("Lutris ({})", self.runner).box_clone()),
        );
        let mut actions = vec![];
        if self.directory.is_some() {
            actions.push("open-dir".to_owned());
        }
        props.insert("actions".to_string(), Variant(actions.box_clone()));
        (
            self.id.to_string(),
            self.name,
            format!("lutris_{}", self.slug),
            30,
            100.0,
            props,
        )
    }
}

fn dbus_err_string<T: std::fmt::Debug>(e: T) -> dbus::MethodErr {
    dbus::MethodErr::failed(&format!("{:?}", e))
}

pub fn serve() -> Result<()> {
    let mut cr = dbus_crossroads::Crossroads::new();
    let token = cr.register("org.kde.krunner1", |b| {
        b.method("Actions", (), ("matches",), |_, _: &mut Lutris, ()| {
            Ok((vec![
                ("open-dir", "Open Directory", "folder-open-symbolic").to_owned()
            ],))
        })
        .annotate("org.qtproject.QtDBus.QtTypeName.Out0", "RemoteActions");
        b.method(
            "Run",
            ("matchId", "actionId"),
            (),
            |_, l: &mut Lutris, (match_id, action): (_, String)| match action.as_str() {
                "open-dir" => l.open_dir(match_id).map_err(dbus_err_string),
                &_ => l.run_game(match_id).map_err(dbus_err_string),
            },
        );
        b.method(
            "Match",
            ("query",),
            ("matches",),
            |_, t: &mut Lutris, (query,)| {
                t.query_game(query)
                    .map(|games| {
                        (games
                            .into_iter()
                            .map(|g| g.into_match())
                            .collect::<Vec<_>>(),)
                    })
                    .map_err(dbus_err_string)
            },
        )
        .annotate("org.qtproject.QtDBus.QtTypeName.Out0", "RemoteMatches");
    });
    cr.insert("/KRunnerLutris", &[token], crate::lutris::Lutris::new()?);
    let conn = dbus::blocking::Connection::new_session()?;
    conn.request_name("org.kde.KRunnerLutris", true, true, true)?;
    cr.serve(&conn)?;
    Ok(())
}
