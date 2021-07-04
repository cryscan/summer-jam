use bevy::prelude::*;

struct Person;

struct Name(String);

fn add_people(mut commands: Commands) {
    commands
        .spawn()
        .insert(Person)
        .insert(Name("Elaina Proctor".to_string()));
    commands
        .spawn()
        .insert(Person)
        .insert(Name("Renzo Hume".to_string()));
    commands
        .spawn()
        .insert(Person)
        .insert(Name("Zayna Nieves".to_string()));
}

fn greet_people(query: Query<&Name, With<Person>>) {
    for name in query.iter() {
        println!("Hello {}!", name.0);
    }
}

fn hello_world() {
    println!("Hello world!");
}

fn main() {
    App::build()
        .add_startup_system(add_people.system())
        .add_system(greet_people.system())
        .add_system(hello_world.system())
        .run();
}
