use crate::blog::config::HIGHLIGHT_THEME;
use crate::blog::error::GeneratorError;
use crate::blog::utils::find_files;
use crate::Config;
use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::plugins::syntect::SyntectAdapter;
use comrak::{
    markdown_to_html_with_plugins, ComrakExtensionOptions, ComrakOptions, ComrakPlugins,
    ComrakRenderOptions,
};
use regex::Regex;
use rexiv2::Metadata;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::fs::{create_dir, remove_dir_all};
use std::io::{Read, Write};
use std::iter::FromIterator;
use std::ops::Index;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use bytebuffer::ByteBuffer;
use tera::{Context, Tera};

const DEFAULT_FILTER: &'static str = ".md";
const KNOWN_ATTRIBUTES: &'static [&str] = &["created", "status", "tag"];
const CUSTOM_POSTS: &'static [&str] = &["recent-posts.md", "overview.md"];
const ESCAPABLE_CHARACTERS: &'static [char] = &[
    '\\', '`', '*', '_', '{', '}', '[', ']', '(', ')', '#', '+', '-', '.', '!',
];

#[derive(Clone, Serialize)]
pub struct Post {
    filename: String,
    tags: Vec<String>,
    created: String,
    status: Option<String>,
    links: Vec<String>,
    images: Vec<String>,
    preview_images: Vec<(String, String)>,
    headline_ids: Vec<String>,
}

#[derive(Serialize)]
struct Headline {
    htype: String,
    title: String,
    id: String,
}

struct ScannerTag {
    name: String,
    value: Option<String>,
    link: Option<String>,
    class: Option<String>,
    is_image: bool,
    pos: (usize, usize),
}

pub struct Generator<'a> {
    instant: Instant,
    last_instant: Instant,
    tera: &'a Tera,
    input_path: PathBuf,
    output_path: PathBuf,
    filter: String,
    markdown_options: ComrakOptions,
    markdown_plugins: ComrakPlugins<'a>,
    headline_regex: Option<Regex>,
    image_regex: Option<Regex>,
    log_buffer: Option<ByteBuffer>,
}

impl<'a> Generator<'a> {
    pub fn new(
        tera: &'a Tera,
        input_path: PathBuf,
        output_path: PathBuf,
        adapter: Option<&'a dyn SyntaxHighlighterAdapter>,
    ) -> Self {
        // configure markdown renderer
        let options = ComrakOptions {
            extension: ComrakExtensionOptions {
                strikethrough: true,
                tagfilter: false,
                table: true,
                autolink: false,
                tasklist: false,
                superscript: false,
                header_ids: None,
                footnotes: true,
                description_lists: false,
                front_matter_delimiter: None,
            },
            parse: Default::default(),
            render: ComrakRenderOptions {
                hardbreaks: false,
                github_pre_lang: false,
                width: 0,
                // required for .has-text-warning on bold text in "De-obfuscating nasty Javascript"
                // *foo*{.has-text-warning} or similar would be nice
                unsafe_: true,
                escape: false,
                list_style: Default::default(),
            },
        };

        let mut generator = Generator {
            instant: Instant::now(),
            last_instant: Instant::now(),
            tera,
            input_path,
            output_path,
            filter: DEFAULT_FILTER.to_string(),
            markdown_options: options,
            markdown_plugins: ComrakPlugins::default(),
            headline_regex: None,
            image_regex: None,
            log_buffer: None,
        };
        generator
            .markdown_plugins
            .render
            .codefence_syntax_highlighter = adapter;
        return generator;
    }

    fn clear_output_path(&self) {
        remove_dir_all(&self.output_path.to_string_lossy().as_ref()).unwrap();
        create_dir(&self.output_path.to_string_lossy().as_ref()).unwrap();
    }

    fn set_filter(&mut self, filter: String) {
        self.filter = filter;
    }

    pub fn log_to_buffer(&mut self) {
        self.log_buffer = Some(ByteBuffer::new());
    }

    pub fn get_log_result(&mut self) -> String {
        if self.log_buffer.is_some() {
            let mut output = String::new();
            let mut log_buffer = self.log_buffer.clone().unwrap();
            let _ = log_buffer.read_to_string(&mut output);
            return output;
        }
        return String::new();
    }

    pub fn generate(&mut self) -> Result<(), GeneratorError> {
        self.log_time(Some("Starting"), true);

        // clear old files
        self.log_time(Some("Clearing output path"), false);
        self.clear_output_path();
        self.log_time(None, false);

        // get all files
        let files = find_files(&self.input_path, Some(self.filter.as_str()));
        self.log_time(Some(format!("Found {} files", &files.len()).as_str()), true);

        let posts: &mut Vec<Post> = &mut vec![];
        let mut custom_post_content = HashMap::new();

        for file in files.into_iter() {
            // read file as string
            let f = fs::read_to_string(format!(
                "{}/{}",
                self.input_path.to_string_lossy(),
                file.name.clone()
            ));
            if f.is_err() {
                return Err(GeneratorError::new(format!(
                    "unable to read file {}: {}",
                    file.name,
                    f.err().unwrap()
                )));
            }
            let file_content = &mut f.unwrap();

            // generate Post struct by parsing markdown manually
            // and manipulate real markdown parser's input (e.g. by removing tags for internal info)
            self.log_time(Some(format!("Parsing post {}", file.name).as_str()), false);
            let mut post = match self.new_post(file.name.clone(), file_content) {
                Ok(post) => post,
                Err(e) => {
                    return Err(GeneratorError::new(format!(
                        "unable to generate post {}: {}",
                        file.name,
                        e.to_string()
                    )))
                }
            };
            self.log_time(None, false);

            // custom posts will be handled manually
            if CUSTOM_POSTS.contains(&file.name.as_str()) {
                custom_post_content.insert(file.name, file_content.to_owned());
                continue;
            }

            let target_filename = self
                .output_path
                .join(Path::new(file.name.replace(".md", ".html").as_str()));
            self.render_and_write(
                file_content.to_string(),
                target_filename,
                None,
                self.generate_extra_context(&post),
                Some(&mut post),
            )?;

            posts.push(post.clone());
        }

        let mut tag_list: Vec<String> = vec![];
        if self.filter == DEFAULT_FILTER {
            // create recent posts
            self.log_time(Some("Generating recent-posts.html"), true);

            posts.sort_by(|a, b| b.created.cmp(&a.created));
            let filtered_posts: &mut Vec<Post> = &mut vec![];
            for post in posts.iter() {
                if post.clone().created.len() > 0 {
                    filtered_posts.push(post.clone());
                }
            }

            let mut context = Context::new();
            context.insert("posts", filtered_posts);
            let recent_posts_html = match self.tera.render("recent-posts.html", &context) {
                Ok(html) => html,
                Err(e) => return Err(GeneratorError::new(e.to_string())),
            };
            let html: String = match custom_post_content.get("recent-posts.md") {
                Some(html) => html.to_owned(),
                None => {
                    return Err(GeneratorError::new(String::from(
                        "custom_post_content not found",
                    )))
                }
            };

            let target_filename = self.output_path.join(Path::new("recent-posts.html"));
            self.render_and_write(
                html.to_string(),
                target_filename,
                Some(&recent_posts_html),
                None,
                None,
            )?;

            // create overview
            self.log_time(Some("Generating overview.html"), true);

            let mut tag_map: HashMap<String, Vec<Post>> = HashMap::new();
            for post in filtered_posts.iter() {
                for post_tag in post.tags.iter() {
                    if !tag_map.contains_key(post_tag) {
                        tag_map.insert(post_tag.to_string(), vec![]);
                        tag_list.push(post_tag.clone());
                    }
                    tag_map.get_mut(post_tag).unwrap().push(post.clone());
                }
            }

            let mut context = Context::new();
            context.insert("tag_map", &tag_map);
            let overview_html = match self.tera.render("overview.html", &context) {
                Ok(html) => html,
                Err(e) => return Err(GeneratorError::new(e.to_string())),
            };
            context.insert("posts", filtered_posts);
            let html: String = match custom_post_content.get("overview.md") {
                Some(html) => html.to_owned(),
                None => {
                    return Err(GeneratorError::new(String::from(
                        "custom_post_content not found",
                    )))
                }
            };

            let target_filename = self.output_path.join(Path::new("overview.html"));
            self.render_and_write(
                html.to_string(),
                target_filename,
                Some(&overview_html),
                None,
                None,
            )?;
        }

        self.log_time(Some("Generating preview images"), false);
        self.generate_preview_images(posts);
        self.log_time(None, false);

        self.log_time(Some("Removing exif data"), false);
        self.remove_exif_data(posts)?;
        self.log_time(None, false);

        if self.filter == DEFAULT_FILTER {
            self.log_time(Some("Verifying links"), false);
            self.verify_links(posts)?;
            self.log_time(None, false);

            self.log_time(Some("Checking unused files"), false);
            self.check_unused_files(posts, &tag_list)?;
            self.log_time(None, false);
        }

        self.log_time(Some("All done!"), true);

        Ok(())
    }

    pub fn generate_preview(&mut self, content: &mut String) -> Result<String, GeneratorError> {
        let mut post = match self.new_post(String::from("preview"), content) {
            Ok(post) => post,
            Err(e) => {
                return Err(GeneratorError::new(format!(
                    "unable to generate post preview: {}",
                    e.to_string()
                )))
            }
        };

        self.render(
            content.to_string(),
            None,
            self.generate_extra_context(&post),
            Some(&mut post),
        )
    }

    fn log_time(&mut self, name: Option<&str>, flush: bool) {
        if self.log_buffer.is_some() {
            let log_buffer = self.log_buffer.as_mut().unwrap();
            if name.is_some() {
                let _ = log_buffer.write_all(format!("{:.2?} {}...", self.instant.elapsed(), name.unwrap()).as_bytes());
                if flush {
                    let _ = log_buffer.write_all("\n".as_bytes());
                }
            } else {
                let _ = log_buffer.write_all(format!(" Done (took {:.2?})\n", self.last_instant.elapsed()).as_bytes());
                self.last_instant = Instant::now();
            }
        } else {
            if name.is_some() {
                print!("{:.2?} {}...", self.instant.elapsed(), name.unwrap());
                if flush {
                    println!();
                }
            } else {
                println!(" Done (took {:.2?})", self.last_instant.elapsed());
                self.last_instant = Instant::now();
            }
        }

    }

    pub fn generate_preview_images(&self, posts: &Vec<Post>) {
        for post in posts {
            for preview_image in post.preview_images.iter() {
                let mut output_path = PathBuf::from(self.output_path.clone());
                output_path.push(preview_image.1.clone());
                if !output_path.exists() {
                    let output_base_path = output_path.parent().unwrap();
                    if !output_base_path.exists() {
                        create_dir(output_base_path).expect(
                            format!(
                                "Unable to create output directory: {}",
                                output_base_path.to_string_lossy()
                            )
                            .as_str(),
                        );
                    }
                    let mut input_path = PathBuf::from(self.output_path.clone());
                    input_path.push(preview_image.0.clone());

                    Command::new("convert")
                        .arg(input_path.to_str().unwrap())
                        .arg("-dither")
                        .arg("Floyd-Steinberg")
                        .arg("-colors")
                        .arg("8")
                        .arg("-quality")
                        .arg("50")
                        .arg("-resize")
                        .arg("500")
                        .arg(output_path.to_str().unwrap())
                        .output()
                        .expect("failed to execute process");
                }
            }
        }
    }

    pub fn remove_exif_data(&self, posts: &Vec<Post>) -> Result<(), GeneratorError> {
        for post in posts {
            for image in post.images.iter() {
                let mut image_path = PathBuf::from(self.output_path.clone());
                image_path.push(image.clone());

                let meta: Metadata = match Metadata::new_from_path(&image_path) {
                    Ok(meta) => meta,
                    Err(e) => {
                        return Err(GeneratorError::new(format!(
                            "Unable to get metadata for {}: {}",
                            image_path.to_string_lossy(),
                            e.to_string()
                        )))
                    }
                };
                if meta.has_exif() {
                    meta.clear_exif();
                    match meta.save_to_file(&image_path) {
                        Ok(_) => {}
                        Err(e) => {
                            return Err(GeneratorError::new(format!(
                                "Unable to clear Exif data: {}",
                                e.to_string()
                            )))
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn verify_links(&self, posts: &Vec<Post>) -> Result<(), GeneratorError> {
        for post in posts {
            'linkLoop: for link in post.links.iter() {
                // skip external links
                if link.starts_with("http") {
                    continue;
                }

                let mut input_path = PathBuf::from(self.input_path.clone());
                input_path.push(link);
                if input_path.exists() {
                    continue;
                }

                let mut url = link.clone();
                let mut headline_id: Option<String> = None;
                match link.split_once('#') {
                    Some(elems) => {
                        url = String::from(elems.0);
                        headline_id = Some(String::from(elems.1));
                    }
                    None => {}
                };

                if url.ends_with(".html") {
                    let mut output_path = PathBuf::from(self.output_path.clone());
                    output_path.push(url.clone());
                    if output_path.exists() {
                        match headline_id {
                            Some(headline_id) => {
                                let md_file_name = url.replace(".html", ".md");
                                for p in posts {
                                    if p.filename != md_file_name {
                                        continue;
                                    }
                                    if p.headline_ids.contains(&headline_id) {
                                        continue 'linkLoop;
                                    }
                                }
                                return Err(GeneratorError::new(format!(
                                    "link not found: {} (unknown headline_id)",
                                    link
                                )));
                            }
                            None => continue,
                        };
                    }
                }
                return Err(GeneratorError::new(format!(
                    "link not found: {} (unknown file)",
                    link
                )));
            }
        }
        Ok(())
    }

    fn check_unused_files(
        &mut self,
        posts: &Vec<Post>,
        tag_list: &Vec<String>,
    ) -> Result<(), GeneratorError> {
        let files = find_files(&self.input_path, None);

        let filtered_files: &mut Vec<String> = &mut vec![];
        for file in files.iter() {
            if file.is_dir || file.name.ends_with(".md") || file.name.ends_with(".html") {
                continue;
            }
            filtered_files.push(file.name.clone());
        }

        for post in posts {
            for link in post.links.iter() {
                // skip external links
                if link.starts_with("http") {
                    continue;
                }

                let handle = &link.replace("../posts/", "");
                // super weird: binary_search and such only operate 9 half-random values
                match search(filtered_files, handle) {
                    Some(i) => {
                        filtered_files.swap_remove(i);
                    }
                    None => {}
                }
            }

            for image in post.images.iter() {
                let handle = &image.replace("../posts/", "");
                match search(filtered_files, handle) {
                    Some(i) => {
                        filtered_files.swap_remove(i);
                    }
                    None => {}
                }
            }

            for preview in post.preview_images.iter() {
                let handle = &preview.1.replace("../posts/", "");
                match search(filtered_files, handle) {
                    Some(i) => {
                        filtered_files.swap_remove(i);
                    }
                    None => {}
                }
            }
        }

        let mut found_overview_files: Vec<String> = vec![];
        for file in filtered_files.iter() {
            if file.starts_with("overview/") {
                for tag in tag_list {
                    if *file == format!("overview/{}_cutout.jpg", tag) {
                        found_overview_files.push(file.to_string());
                    }
                }
            }
        }
        for file in found_overview_files {
            match search(filtered_files, &file) {
                Some(i) => {
                    filtered_files.swap_remove(i);
                }
                None => {}
            }
        }

        if filtered_files.len() > 0 {
            let log_buffer = self.log_buffer.as_mut().unwrap();
            let _ = log_buffer.write_all(format!("\nfound {} entries:\n", filtered_files.len()).as_bytes());
            for file in filtered_files.iter() {
                let _ = log_buffer.write_all(format!("{}\n", file).as_bytes());
            }
        }

        Ok(())
    }

    fn generate_extra_context(&self, post: &Post) -> Option<Context> {
        let mut features: Vec<&str> = vec![];
        if !post.preview_images.is_empty() {
            features.push("preview");
        }

        for link in &post.links {
            if link.ends_with(".stl") {
                features.push("3d");
                break;
            }
        }

        if features.len() > 0 || post.status.is_some() {
            let mut context = Context::new();

            if features.len() > 0 {
                context.insert("features", &features);
            }

            if post.status.is_some() {
                context.insert("status", &post.status.as_ref().unwrap());
            }

            return Some(context);
        }

        return None;
    }

    fn render(
        &mut self,
        file_content: String,
        html_append: Option<&String>,
        extra_context: Option<Context>,
        post: Option<&mut Post>,
    ) -> Result<String, GeneratorError> {
        // convert to markdown
        self.log_time(Some("Rendering markdown"), false);
        let mut md = markdown_to_html_with_plugins(
            file_content.as_str(),
            &self.markdown_options,
            &self.markdown_plugins,
        );
        self.log_time(None, false);

        // append manually rendered html
        if html_append.is_some() {
            md.push_str(html_append.unwrap().to_owned().as_str());
        }

        self.log_time(Some("Replacing preview images"), false);
        // find all images
        if self.image_regex.is_none() {
            self.image_regex = match Regex::new(r#"(?m)<img src="(.+?)" alt="(.+?)" />"#) {
                Ok(re) => Some(re),
                Err(e) => return Err(GeneratorError::new(format!("Unable to build regex: {}", e))),
            };
        }
        let re = self.image_regex.clone().unwrap();

        // replace preview images
        for cap in re.captures_iter(md.clone().as_str()) {
            let from = format!("<img src=\"{}\" alt=\"{}\" />", &cap[1], &cap[2]);
            let to = format!(
                "<a href=\"{}\" class=\"preview-image\"><img src=\"{}\" alt=\"{}\" /></a>",
                &cap[1].replace("/preview/", "/"),
                &cap[1],
                &cap[2]
            );
            let start_pos = match md.find(&from) {
                Some(start_pos) => start_pos,
                None => {
                    return Err(GeneratorError::new(String::from(
                        "Image starting position not found",
                    )))
                }
            };
            md.replace_range(start_pos..start_pos + from.len(), to.as_str());
        }
        self.log_time(None, false);

        self.log_time(Some("Modifying headlines"), false);
        // find all headlines
        if self.headline_regex.is_none() {
            self.headline_regex = match Regex::new(r"(?m)^<h([1-6])>(.+?)</h([1-6])>$") {
                Ok(re) => Some(re),
                Err(e) => return Err(GeneratorError::new(format!("Unable to build regex: {}", e))),
            };
        }
        let re = self.headline_regex.clone().unwrap();
        let mut headlines: Vec<Headline> = vec![];
        let mut headline_ids: Vec<String> = vec![];
        for cap in re.captures_iter(md.as_str()) {
            if &cap[1] != &cap[3] {
                return Err(GeneratorError::new(String::from(
                    "Unmatching headline tags found",
                )));
            }
            let id = self.title2id(cap[2].to_string());
            headline_ids.push(id.clone());
            headlines.push(Headline {
                htype: cap[1].to_string(),
                title: cap[2].to_string(),
                id,
            });
        }

        // write to original Post if available
        if post.is_some() {
            post.unwrap().headline_ids = headline_ids;
        }

        // add id to each headline
        for headline in headlines.iter() {
            let from = format!(
                "<h{}>{}</h{}>",
                headline.htype, headline.title, headline.htype
            );
            let to = format!(
                "<h{} id=\"{}\">{}</h{}>",
                headline.htype, headline.id, headline.title, headline.htype
            );
            let start_pos = match md.find(&from) {
                Some(start_pos) => start_pos,
                None => {
                    return Err(GeneratorError::new(String::from(
                        "Headline starting position not found",
                    )))
                }
            };
            md.replace_range(start_pos..start_pos + from.len(), to.as_str());
        }

        let filtered_headlines: Vec<Headline> = headlines
            .into_iter()
            .filter(|headline| headline.htype != "1")
            .collect();
        self.log_time(None, false);

        self.log_time(Some("Rendering HTML"), false);
        // render html
        let mut context = Context::new();
        context.insert("content", &md);
        if filtered_headlines.len() > 0 {
            context.insert("headlines", &filtered_headlines);
        }
        if extra_context.is_some() {
            context.extend(extra_context.unwrap());
        }
        let html = match self.tera.render("post.html", &context) {
            Ok(html) => html,
            Err(e) => return Err(GeneratorError::new(e.to_string())),
        };
        self.log_time(None, false);

        Ok(html)
    }

    fn render_and_write(
        &mut self,
        file_content: String,
        target_filename: PathBuf,
        html_append: Option<&String>,
        extra_context: Option<Context>,
        post: Option<&mut Post>,
    ) -> Result<(), GeneratorError> {
        let html = self.render(file_content, html_append, extra_context, post)?;

        self.log_time(Some("Writing HTML"), false);
        // write html
        match fs::write(target_filename.clone(), html) {
            Ok(_) => {}
            Err(e) => {
                return Err(GeneratorError::new(format!(
                    "unable to write file {}: {}",
                    target_filename.to_string_lossy(),
                    e.to_string()
                )));
            }
        }
        self.log_time(None, false);

        Ok(())
    }

    pub fn new_post(
        &self,
        filename: String,
        file_content: &mut String,
    ) -> Result<Post, GeneratorError> {
        let mut prev_char = '\0';
        let mut prev_pos: usize = 0;

        let mut is_tag_open = false;
        let mut tag_open_pos: usize = 0;
        let mut tag_close_pos: usize = 0;

        let mut is_link_open = false;

        let mut is_image = false;

        let mut is_class_open = false;

        let mut is_code_opening = false;
        let mut is_code_possible_closing = false;
        let mut is_multiline_code = false;
        let mut code_open_count: usize = 0;

        let mut tag_buf: Vec<char> = Vec::new();
        let mut link_buf: Vec<char> = Vec::new();
        let mut class_buf: Vec<char> = Vec::new();

        let mut tags: Vec<ScannerTag> = Vec::new();

        const ESCAPE_CHAR: char = '\\';

        for (pos, char) in file_content.char_indices() {
            // update prev_char now, so it won't be forgotten
            // _prev_char will be used from now on
            let _prev_char = prev_char;
            prev_char = char;
            prev_pos = pos;

            // ignore escaped characters
            if _prev_char == ESCAPE_CHAR && ESCAPABLE_CHARACTERS.contains(&char) {
                continue;
            }

            // detect code block opening/closing
            if char == '`' {
                if _prev_char == '`' {
                    if is_code_opening {
                        // multi line code opening
                        code_open_count += 1;
                        is_multiline_code = true;
                    } else {
                        // multi line code closing
                        if is_code_possible_closing {
                            // remove twice after being skipped on previous iteration
                            code_open_count -= 1;
                            is_code_possible_closing = false;
                        }
                        code_open_count -= 1;
                    }
                } else {
                    if code_open_count < 1 {
                        // single code opening
                        is_code_opening = true;
                        code_open_count += 1;
                    } else {
                        // single code closing
                        if is_multiline_code {
                            // wait for removal until next iteration
                            is_code_possible_closing = true;
                            continue;
                        }
                        code_open_count -= 1;
                    }
                }

                if code_open_count > 3 {
                    return Err(GeneratorError::new(format!(
                        "too many code block ticks in {} pos {}",
                        filename, pos
                    )));
                } else if code_open_count == 0 {
                    is_multiline_code = false;
                }

                continue;
            }

            // skipping for code blocks
            if code_open_count > 0 {
                // until this point, the code block must be open
                is_code_opening = false;
                is_code_possible_closing = false;

                // when code block is open, skip until closed
                continue;
            }

            // register opening square bracket
            if char == '[' {
                if is_tag_open {
                    return Err(GeneratorError::new(format!(
                        "tag opened twice in {} pos {}",
                        filename, pos
                    )));
                }
                is_tag_open = true;
                tag_open_pos = pos;

                // if preceded by bang, it's an image
                if _prev_char == '!' {
                    is_image = true;
                    tag_open_pos -= 1;
                }

                continue;
            }

            // register closing square bracket
            if char == ']' {
                is_tag_open = false;
                tag_close_pos = pos;
                continue;
            }

            // if square bracket just closed
            if _prev_char == ']' {
                // if round bracket opens, it's a link or image
                if char == '(' {
                    if is_link_open {
                        return Err(GeneratorError::new(format!(
                            "link opened twice in {} pos {}",
                            filename, pos
                        )));
                    }
                    is_link_open = true;
                    continue;
                } else {
                    // no round bracket means it's a simple tag
                    let (name, value) = self.get_tag_name_value(String::from_iter(tag_buf.iter()));
                    tags.push(ScannerTag {
                        name,
                        value,
                        link: None,
                        class: None,
                        is_image: false,
                        pos: (tag_open_pos, tag_close_pos + 1),
                    });
                    tag_buf.clear();
                    is_image = false;
                }
            }

            // if link or image is closed
            if is_link_open && _prev_char == ')' {
                is_link_open = false;

                // check for curly bracket to watch for class
                if char == '{' {
                    is_class_open = true;
                } else {
                    // if no class found, then the tag can be added as link or image
                    let (name, value) = self.get_tag_name_value(String::from_iter(tag_buf.iter()));
                    tags.push(ScannerTag {
                        name,
                        value,
                        link: Some(String::from_iter(link_buf.iter())),
                        class: None,
                        is_image,
                        pos: (tag_open_pos, pos),
                    });
                    tag_buf.clear();
                    link_buf.clear();
                    is_image = false;
                }

                continue;
            }

            // register link or image with class
            if is_class_open && char == '}' {
                is_class_open = false;

                let (name, value) = self.get_tag_name_value(String::from_iter(tag_buf.iter()));
                tags.push(ScannerTag {
                    name,
                    value,
                    link: Some(String::from_iter(link_buf.iter())),
                    class: Some(String::from_iter(class_buf.iter())),
                    is_image,
                    pos: (tag_open_pos, pos + 1),
                });
                tag_buf.clear();
                link_buf.clear();
                class_buf.clear();
                is_image = false;
            }

            if is_tag_open {
                tag_buf.push(char);
            }

            if is_link_open && char != ')' {
                link_buf.push(char);
            }

            if is_class_open {
                class_buf.push(char);
            }
        }

        if prev_char == ']' {
            return Err(GeneratorError::new(format!(
                "possible late tag closing in {}",
                filename
            )));
        }

        // catch late link closings
        // required because _prev_char is never matched
        if is_link_open && prev_char == ')' {
            let (name, value) = self.get_tag_name_value(String::from_iter(tag_buf.iter()));
            tags.push(ScannerTag {
                name,
                value,
                link: Some(String::from_iter(link_buf.iter())),
                class: None,
                is_image,
                pos: (tag_open_pos, prev_pos),
            });
        }

        // late sanity check
        if code_open_count != 0 {
            return Err(GeneratorError::new(format!(
                "code_open_count is not 0 for {}",
                filename
            )));
        }

        let mut post = Post {
            filename: filename.clone(),
            tags: vec![],
            created: "".to_string(),
            status: None,
            links: vec![],
            images: vec![],
            preview_images: vec![],
            headline_ids: vec![],
        };
        let mut char_shift_pos: usize = 0;
        for tag in tags.iter() {
            // only react to specific tags
            if KNOWN_ATTRIBUTES.contains(&tag.name.as_str()) {
                match tag.name.as_str() {
                    "created" => post.created = tag.value.clone().unwrap(),
                    "tag" => post.tags.push(tag.value.clone().unwrap()),
                    "status" => post.status = Some(tag.value.clone().unwrap()),
                    _ => {
                        return Err(GeneratorError::new(format!(
                            "found known attribute without handler '{}' in {}",
                            tag.name, filename
                        )))
                    }
                }

                // if tag ends with newline, remove newline as well
                let range_expand = {
                    if file_content
                        .index(tag.pos.1 - char_shift_pos..tag.pos.1 + 1 - char_shift_pos)
                        == "\n"
                    {
                        1
                    } else {
                        0
                    }
                };

                // remove tag from markdown
                file_content.replace_range(
                    tag.pos.0 - char_shift_pos..tag.pos.1 + range_expand - char_shift_pos,
                    "",
                );

                // calculate position correction
                char_shift_pos += (tag.pos.1 + range_expand) - tag.pos.0;
            } else if tag.is_image {
                post.images.push(tag.link.clone().unwrap());

                if tag.class.is_some() && tag.class.as_ref().unwrap() == "preview" {
                    let old_section = file_content
                        .get(tag.pos.0 - char_shift_pos..tag.pos.1 - char_shift_pos)
                        .unwrap();
                    let mut without_tag = old_section.clone().replace("{preview}", "");
                    let last_slash_pos = without_tag.rfind('/').unwrap();
                    without_tag.insert_str(last_slash_pos, "/preview");
                    let length_diff = old_section.len() - without_tag.len();

                    post.preview_images.push((
                        tag.link.clone().unwrap(),
                        self.to_preview_image_url(tag.link.as_ref().unwrap().to_string()),
                    ));

                    // replace image with preview image
                    file_content.replace_range(
                        tag.pos.0 - char_shift_pos..tag.pos.1 - char_shift_pos,
                        without_tag.as_str(),
                    );

                    // calculate position correction
                    char_shift_pos += length_diff;
                }
            } else if tag.link.is_some() {
                post.links.push(tag.link.clone().unwrap());
            } else if tag.link.is_none() && tag.value.is_some() {
                return Err(GeneratorError::new(format!(
                    "unknown attribute '{}' in {}",
                    tag.name, filename
                )));
            }
        }

        Ok(post)
    }

    fn get_tag_name_value(&self, input: String) -> (String, Option<String>) {
        match input.split_once(":") {
            Some(values) => (values.0.to_string(), Some(values.1.to_string())),
            None => (input.clone(), None),
        }
    }

    fn to_preview_image_url(&self, url: String) -> String {
        let parts = url.rsplit_once('/').unwrap();
        return format!("{}/preview/{}", parts.0, parts.1);
    }

    fn title2id(&self, title: String) -> String {
        title.replace(" ", "_")
    }
}

fn search(haystack: &Vec<String>, needle: &String) -> Option<usize> {
    for (pos, elem) in haystack.iter().enumerate() {
        if elem == needle {
            return Some(pos);
        }
    }
    None
}

pub fn generate_all(config: &Config, tera: &Tera) -> Result<(), GeneratorError> {
    generate_files(config, tera, None)
}

pub fn generate_files(
    config: &Config,
    tera: &Tera,
    name_arg: Option<&String>,
) -> Result<(), GeneratorError> {
    let adapter = SyntectAdapter::new(HIGHLIGHT_THEME);
    let mut generator = Generator::new(
        tera,
        config.get_input_path(),
        config.get_output_path(),
        Some(&adapter),
    );

    // set default for name filter
    if name_arg.is_some() {
        generator.set_filter(name_arg.unwrap().to_string());
    }

    generator.generate()
}
