use embed_resource;

fn main() {
    embed_resource::compile("resources/resources.rc", embed_resource::NONE);
    slint_build::compile("resources/ui/window.slint").unwrap();
}
