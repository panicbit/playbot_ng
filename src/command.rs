pub struct Command<'msg> {
    name: &'msg str,
    args: Vec<&'msg str>,
    raw_args: &'msg str,
}

impl<'msg> Command<'msg> {
    pub fn parse(prefix: &str, msg: &'msg str) -> Option<Self> {
        let msg = msg.trim();

        if !msg.starts_with(prefix) {
            return None;
        }

        let msg = &msg[prefix.len()..];
        let mut parts = msg.splitn(2, ' ');
        let name = parts.next()?;
        let raw_args = parts.next().unwrap_or("");

        Some(Command {
            name: name,
            args: raw_args.split_whitespace().collect(),
            raw_args,
        })
    }

    pub fn name(&self) -> &'msg str {
        self.name
    }
    
    pub fn args(&self) -> &[&'msg str] {
        &self.args
    }

    pub fn raw_args(&self) -> &'msg str {
        &self.raw_args
    }
}
