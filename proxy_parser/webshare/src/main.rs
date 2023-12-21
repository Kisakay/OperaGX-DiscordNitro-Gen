use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

fn main() -> io::Result<()> {
    // Ouvrir le fichier d'entrée
    let input_file = File::open("../input.txt")?;
    let reader = BufReader::new(input_file);

    // Ouvrir le fichier de sortie
    let mut output_file = File::create("../output.txt")?;

    // Parcourir chaque ligne du fichier d'entrée
    for line in reader.lines() {
        let proxy = line?;

        // Diviser la ligne en parties (ip, port, username, password)
        let parts: Vec<&str> = proxy.split(':').collect();

        if parts.len() == 4 {
            // Construire la nouvelle chaîne au format demandé
            let new_proxy = format!(
                "{}:{}@{}:{}\n",
                parts[2], parts[3], parts[0], parts[1]
            );

            // Écrire la nouvelle chaîne dans le fichier de sortie
            output_file.write_all(new_proxy.as_bytes())?;
        }
    }

    Ok(())
}
