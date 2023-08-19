use std::path::Path;

pub fn read<'a, 'b>(
    name: impl Into<&'a str>,
    file: impl Into<&'b str>,
) -> Result<String, std::io::Error> {
    let path = Path::new("/etc/secrets/")
        .join(name.into())
        .join(file.into());

    std::fs::read_to_string(path)
}
