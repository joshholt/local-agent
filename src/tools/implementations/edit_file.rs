use ollama_rs::generation::tools::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use std::error::Error;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Create a file at a given path and writes the given contents.
///
/// * path - The path to the file to create.
/// * contents - The contents to write to the give file.
async fn create_file(path: &PathBuf, contents: &str) -> Result<String, Box<dyn Error>> {
    match File::create(path).await {
        Ok(mut file) => {
            if let Err(e) = file.write_all(contents.as_bytes()).await {
                return Err(format!(
                    "Failed to write to file at {}, Reason: {}",
                    path.display(),
                    e
                )
                .into());
            } else {
                if let Err(e) = file.flush().await {
                    return Err(format!(
                        "Failed to flush contents to file at {}, Reason: {}",
                        path.display(),
                        e
                    )
                    .into());
                } else {
                    return Ok(format!("Successfully created file at {}", path.display()).into());
                }
            }
        }
        Err(e) => {
            return Err(
                format!("Failed to create file at {}, Reason: {}", path.display(), e).into(),
            );
        }
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(description = "The path to the file.")]
    path: String,

    #[schemars(
        description = "Text to search for - must match eactly and must have only one match exactly."
    )]
    old_str: String,

    #[schemars(description = "Text to replace old_str with.")]
    new_str: String,
}

pub struct EditFile {}

impl Tool for EditFile {
    type Params = Params;

    fn name() -> &'static str {
        "edit_file"
    }

    fn description() -> &'static str {
        "Make edits to a text file. Replaces 'old_str' with 'new_str' in the given file.
'old_str' and 'new_str' Must be different from each other.
If the file specified doesn't exist, it will be created."
    }

    async fn call(
        &mut self,
        parameters: Self::Params,
    ) -> Result<String, Box<dyn Error + Sync + Send>> {
        let path = PathBuf::from(&parameters.path);

        // If the file specified doesn't exits then it needs to be created
        // If the path for the file is not in the CWD and old_str is nil/empty then the directory structure will need to be created first.
        // Then the file can be created at the path given with the contents of new_str
        // If the file path given doesn't exist but is in the CWD and old_str is empty then we can just write the new file with the contens of new_str in CWD.
        // If the file exists and old_str and new_str are not empty and the result of replacing old_str with new_str doesn't yield the original contents of the files,
        //    then we write the replaced content back to the existing file.
        // Unless there are errors we return the string OK, otherwise we return Err with the reason as a string.
        if path.display().to_string().is_empty() || &parameters.old_str == &parameters.new_str {
            return Err(format!("Could not edit a file, input parameters were invalid.").into());
        }

        match fs::metadata(&path).await {
            Ok(_) => {
                // 1. Read the contents of the file
                let mut reader = fs::OpenOptions::new().read(true).open(&path).await?;
                let mut contents = String::new();
                reader.read_to_string(&mut contents).await?;

                // 2. Replace all old_str with new_str
                let new_contents = contents.replace(&parameters.old_str, &parameters.new_str);
                // 3. Check to see if new_contents is equal to old contents and old_str is not empty
                if contents == new_contents && !parameters.old_str.is_empty() {
                    // 3a.   Return Err old_str not found in file.
                    return Err(format!("old_str was not found in the file.").into());
                }
                // 4. Write new_contents to the files, if no errors occur writing to the file, return Ok with the str "OK"
                match fs::write(&path, new_contents.as_bytes()).await {
                    // 4a.   If an error occurs writing the file return Err with the reason.
                    Ok(_) => {
                        return Ok("OK".to_string());
                    }
                    Err(e) => {
                        return Err(format!(
                            "Failed to edit file at {}, reason: {}",
                            path.display(),
                            e
                        )
                        .into());
                    }
                }
            }
            Err(_) => {
                // 1a. If it is CWD then we can create the new file with the contents of new_str
                //       If there is an error creating the file, return Err with the string "failed to create file, {}", err
                //       Else return Ok with the string "Successfully created file {}", path
                // 1b. if it is not CWD then we have to create the dir structure first, then write the file with the contents of new_str
                //       If there is an error creating the directory structure, return Err with string "failed to create directory {}", err
                //       Else move on to writing the file above.
                // Since the files at the given path doesn't not exist we need to create it.
                // 1. Check the parent of the file path given
                if let Some(parent) = path.parent() {
                    // We have a parent so we aren't working on a file in CWD.
                    match fs::create_dir_all(&parent).await {
                        Ok(_) => match create_file(&path, &parameters.new_str).await {
                            Ok(s) => {
                                return Ok(s);
                            }
                            Err(e) => {
                                return Err(format!("{}", e).into());
                            }
                        },
                        Err(e) => {
                            return Err(format!("Failed to create required directory structure for new file. Reason: {}", e).into());
                        }
                    }
                } else {
                    match create_file(&path, &parameters.new_str).await {
                        Ok(s) => {
                            return Ok(s);
                        }
                        Err(e) => {
                            return Err(format!("{}", e).into());
                        }
                    }
                }
            }
        }
    }
}
