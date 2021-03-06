use serde::Serialize;

extern crate rand;

use rand::Rng;

use crate::{AlertType, OutParams, Source};
use concourse_resource::BuildMetadata;

#[derive(Serialize)]
pub struct Message {
    pub color: String,
    pub text: Option<String>,
    pub icon_url: String,
    pub image_url: String,
}

struct FormattedBuildInfo {
    job_name: String,
    build_name: String,
    build_number: String,
    build_url: Option<String>,
}

fn formatted_build_info_from_params(build_metadata: &BuildMetadata) -> FormattedBuildInfo {
    if let (Some(pipeline_name), Some(job_name), Some(name)) = (
        build_metadata.pipeline_name.as_ref(),
        build_metadata.job_name.as_ref(),
        build_metadata.name,
    ) {
        FormattedBuildInfo {
            job_name: format!("{}/{}", pipeline_name, job_name),
            build_name: format!("{}/{} #{}", pipeline_name, job_name, name,),
            build_number: format!("#{}", name),
            build_url: Some(format!(
                "{}/teams/{}/pipelines/{}/jobs/{}/builds/{}",
                build_metadata.atc_external_url,
                urlencoding::encode(&build_metadata.team_name),
                urlencoding::encode(&pipeline_name),
                urlencoding::encode(&job_name),
                name,
            )),
        }
    } else {
        FormattedBuildInfo {
            job_name: String::from("unknown job"),
            build_name: String::from("unknown build"),
            build_number: String::from("unknown build"),
            build_url: None,
        }
    }
}

fn find_random_string() -> String
{
    let  pictures = vec!["https://1.bp.blogspot.com/-Av-RYG5DXLk/XU3nZMLR9yI/AAAAAAAATVA/16f5apNqph4q9K1Z_U6-J6IbnwUFI_togCLcBGAs/s640/rambo%2B3.jpg","https://1.bp.blogspot.com/-s2AlGmyUQmM/XU3m_5JgUBI/AAAAAAAATU4/jUj7T8eCgwk_cSubSLDpSj7EGNESfD9-gCLcBGAs/s640/rambo%2B2.webp", "https://static.kino.de/wp-content/uploads/2019/10/rambo-i-iii-1987-film-rcm1024x512u.jpg", "https://wegotthiscovered.com/wp-content/uploads/2018/05/rambo-1-670x335.jpg", "https://midnightmovietrain.files.wordpress.com/2014/09/rambo-iv-2.jpg"];
    let mut rng = rand::thread_rng();
    pictures[rng.gen_range(0, 5)].to_string()

}

fn find_channel(branch: Option<String>, source: &Source) -> String {
    let mut s = String::new();

    match branch.unwrap().as_str() {
        "integration" => {
            s = (&source.integration).parse().unwrap();
        },
        "production" => {
            s = (&source.production).parse().unwrap();
        },

        "staging" => {
            s = (&source.staging).parse().unwrap();
        },
        "hotfix" => {
            s = (&source.hotfix).parse().unwrap();
        },
        _ => {},
    }
    s
}

impl Message {
    pub(crate) fn new(params: &OutParams, input_path: &str) -> Message {
        let mut message = match params.alert_type {
            AlertType::Success | AlertType::Fixed => Message {
                color: String::from("#11c560"),
                icon_url: String::from(
                    "https://ci.concourse-ci.org/public/images/favicon-succeeded.png",
                ),
                text: None,
                image_url: String::from(find_random_string()),
            },
            AlertType::Failed | AlertType::Broke => Message {
                color: String::from("#ed4b35"),
                icon_url: String::from(
                    "https://ci.concourse-ci.org/public/images/favicon-failed.png",
                ),
                text: None,
                image_url: String::from(find_random_string()),
            },
            AlertType::Started => Message {
                color: String::from("#fad43b"),
                icon_url: String::from(
                    "https://ci.concourse-ci.org/public/images/favicon-started.png",
                ),
                text: None,
                image_url: String::from(find_random_string()),
            },
            AlertType::Aborted => Message {
                color: String::from("#8b572a"),
                icon_url: String::from(
                    "https://ci.concourse-ci.org/public/images/favicon-aborted.png",
                ),
                text: None,
                image_url: String::from(find_random_string()),
            },
            AlertType::Errored => Message {
                color: String::from("#f5a623"),
                icon_url: String::from(
                    "https://ci.concourse-ci.org/public/images/favicon-errored.png",
                ),
                text: None,
                image_url: String::from(find_random_string()),
            },
            AlertType::Custom => Message {
                color: String::from("#35495c"),
                icon_url: String::from(
                    "https://ci.concourse-ci.org/public/images/favicon-pending.png",
                ),
                text: None,
                image_url: String::from(find_random_string()),
            },
        };
        if let Some(color) = params.color.as_ref() {
            message.color = color.clone();
        }
        match (
            params.message_file.as_ref(),
            params.message.as_ref(),
            params.fail_if_message_file_missing,
        ) {
            (Some(file), Some(text), _) => {
                let mut path = std::path::PathBuf::new();
                path.push(input_path);
                path.push(file);
                message.text = Some(std::fs::read_to_string(path).unwrap_or_else(|_| text.clone()));
            }
            (Some(file), None, true) => {
                let mut path = std::path::PathBuf::new();
                path.push(input_path);
                path.push(file);
                message.text = Some(std::fs::read_to_string(path).expect("error reading file"));
            }
            (Some(file), None, false) => {
                let mut path = std::path::PathBuf::new();
                path.push(input_path);
                path.push(file);
                message.text = Some(
                    std::fs::read_to_string(path)
                        .unwrap_or_else(|_| format!("error reading file {}", file)),
                );
            }
            (None, Some(text), _) => {
                message.text = Some(text.clone());
            }
            (None, None, _) => {}
        }
        if params.message_as_code {
            message.text = message.text.map(|text| format!("```{}```", text));
        }
        message
    }

    pub(crate) fn into_slack_message(
        self,
        build_metadata: BuildMetadata,
        params: &OutParams, source: &Source
    ) -> slack_push::Message {
        let formatted_build_info = formatted_build_info_from_params(&build_metadata);
        slack_push::Message {
            attachments: Some(vec![slack_push::message::Attachment {
                author_name: match params.mode {
                    crate::Mode::Concise => {
                        Some(self.text.clone().unwrap_or(formatted_build_info.build_name))
                    }
                    crate::Mode::Normal | crate::Mode::NormalWithInfo => Some(format!(
                        "{} - {}",
                        formatted_build_info.build_name,
                        params.alert_type.message()
                    )),
                },
                text: match params.mode {
                    crate::Mode::Concise => None,
                    crate::Mode::Normal | crate::Mode::NormalWithInfo => self.text,
                },
                mrkdwn_in: Some(vec![String::from("text")]),
                color: Some(self.color),
                footer: formatted_build_info.build_url,
                footer_icon: Some(self.icon_url),
                thumb_url: Some(self.image_url),
                fields: match params.mode {
                    crate::Mode::Concise | crate::Mode::Normal => None,
                    crate::Mode::NormalWithInfo => Some(vec![
                        slack_push::message::AttachmentField {
                            title: Some(String::from("Job")),
                            value: Some(formatted_build_info.job_name),
                            short: Some(true),
                        },
                        slack_push::message::AttachmentField {
                            title: Some(String::from("Build")),
                            value: Some(formatted_build_info.build_number),
                            short: Some(true),
                        },

                    ]),
                },
                ..Default::default()
            }]),
            channel: Option::from(find_channel(params.channel.clone(), source)),

            ..Default::default()
        }
    }
}
