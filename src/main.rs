use std::collections::{HashMap, HashSet};
use std::io::BufRead;
use std::{error, fs, io, path, result};

use lazy_static::lazy_static;
use regex::Regex;

fn read_lines<P: AsRef<path::Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>> {
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

lazy_static! {
    static ref RULE_REGEX: Regex = Regex::new(r"^(?P<color>(\w+\s?)+) bags contain (?P<contents>.+)$").expect("illegal regex");
    static ref CONTENT_REGEX: Regex = Regex::new(r"^\s?(?P<count>\d+)\s(?P<color>(\w+\s?)+)\sbags?\.?").expect("illegal regex");
}

#[derive(Eq, PartialEq, Hash)]
struct Content {
    count: u64,
    color: String,
}

struct Bag {
    color: String,
    contents: Vec<Content>,
}

impl Content {
    fn new_from_rule(rule: &str) -> Option<Content> {
        CONTENT_REGEX.captures(rule).map(|caps| Content {
            count: caps["count"].parse::<u64>().expect("invalid contents"),
            color: caps["color"].to_string(),
        })
    }
}

impl Bag {
    fn new_from_rule(rule: &str) -> Bag {
        let caps = RULE_REGEX.captures(rule).expect("invalid rule");
        let color = caps["color"].to_string();
        let contents: Vec<Content> =
            caps["contents"]
                .split(',')
                .fold(Vec::new(), |mut all, rule| {
                    if let Some(content) = Content::new_from_rule(rule) {
                        all.push(content);
                    }
                    all
                });
        Bag { color, contents }
    }
}

fn to_bags(lines: impl Iterator<Item = result::Result<String, io::Error>>) -> Vec<Bag> {
    let mut bags: Vec<Bag> = Vec::new();
    for line_res in lines {
        if let Ok(line) = line_res {
            bags.push(Bag::new_from_rule(&line))
        }
    }
    bags
}

fn bags_to_contained_by_graph(bags: &[Bag]) -> HashMap<String, HashSet<String>> {
    let mut graph = HashMap::new();
    for bag in bags {
        for contents in bag.contents.iter() {
            if !graph.contains_key(&contents.color) {
                graph.insert(contents.color.clone(), HashSet::new());
            }
            let contained_by = graph.get_mut(&contents.color).unwrap();
            contained_by.insert(bag.color.clone());
        }
    }
    graph
}

fn bags_to_contains_graph(bags: &[Bag]) -> HashMap<&str, HashSet<&Content>> {
    let mut graph = HashMap::new();
    for bag in bags {
        let mut set = HashSet::new();
        for contents in bag.contents.iter() {
            set.insert(contents);
        }
        graph.insert(bag.color.as_str(), set);
    }
    graph
}

fn find_potential_containers(
    color: &str,
    graph: &HashMap<String, HashSet<String>>,
) -> HashSet<String> {
    let mut containers = HashSet::new();
    _find_potential_containers(color, graph, &mut containers);
    containers
}

fn _find_potential_containers(
    color: &str,
    graph: &HashMap<String, HashSet<String>>,
    containers: &mut HashSet<String>,
) {
    if let Some(contained_by) = graph.get(color) {
        for color in contained_by {
            if !containers.contains(color) {
                containers.insert(color.to_string());
                _find_potential_containers(color, graph, containers);
            }
        }
    }
}

fn find_bag_count(color: &str, graph: &HashMap<&str, HashSet<&Content>>) -> u64 {
    // -1 because the outer bag doesn't count
    _find_bag_count(color, graph) - 1u64
}
fn _find_bag_count(color: &str, graph: &HashMap<&str, HashSet<&Content>>) -> u64 {
    let contents = graph.get(color).unwrap();
    if !contents.is_empty() {
        contents
            .iter()
            .map(|content| content.count * _find_bag_count(&content.color, graph))
            .sum::<u64>()
            + 1u64
    } else {
        1
    }
}

fn main() -> result::Result<(), Box<dyn error::Error>> {
    let lines = read_lines("input.txt")?;
    let bags = to_bags(lines);
    let graph = bags_to_contained_by_graph(&bags);
    let can_contain = find_potential_containers("shiny gold", &graph);

    println!(
        "The number of potential containers is {}",
        can_contain.len()
    );

    let contains_graph = bags_to_contains_graph(&bags);
    let count = find_bag_count("shiny gold", &contains_graph);
    println!("You have to buy {} bags", count);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    const TEST_RULES: &'static str = indoc! {"\
        light red bags contain 1 bright white bag, 2 muted yellow bags.
        dark orange bags contain 3 bright white bags, 4 muted yellow bags.
        bright white bags contain 1 shiny gold bag.
        muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.
        shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.
        dark olive bags contain 3 faded blue bags, 4 dotted black bags.
        vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.
        faded blue bags contain no other bags.
        dotted black bags contain no other bags."};

    const ALTERNATE_TEST_RULES: &'static str = indoc! {"\
        shiny gold bags contain 2 dark red bags.
        dark red bags contain 2 dark orange bags.
        dark orange bags contain 2 dark yellow bags.
        dark yellow bags contain 2 dark green bags.
        dark green bags contain 2 dark blue bags.
        dark blue bags contain 2 dark violet bags.
        dark violet bags contain no other bags."};

    #[test]
    fn bag_has_correct_color_and_contents() {
        let bag = Bag::new_from_rule(
            "muted lime bags contain 1 wavy lime bag, 1 vibrant green bag, 3 light yellow bags.",
        );
        assert_eq!("muted lime", bag.color);
        let wavy_lime = bag
            .contents
            .iter()
            .find(|content| content.color == "wavy lime")
            .expect("no wavy lime bag");
        assert_eq!(1, wavy_lime.count);
        let vibrant_green = bag
            .contents
            .iter()
            .find(|content| content.color == "vibrant green")
            .expect("no vibrant green");
        assert_eq!(1, vibrant_green.count);
        let light_yellow = bag
            .contents
            .iter()
            .find(|content| content.color == "light yellow")
            .expect("no light yellow");
        assert_eq!(3, light_yellow.count);
    }

    fn to_line_results(data: &'static str) -> impl Iterator<Item = io::Result<String>> {
        data.split('\n').map(|s| Ok(s.to_string()))
    }

    #[test]
    fn bag_has_correct_color_but_no_contents() {
        let bag = Bag::new_from_rule("dotted teal bags contain no other bags.");
        assert_eq!("dotted teal", bag.color);
        assert!(bag.contents.is_empty());
    }

    #[test]
    fn converts_rules_to_bags() {
        let bags = to_bags(to_line_results(TEST_RULES));
        assert_eq!(9, bags.len());
    }

    #[test]
    fn converts_bags_to_contained_by_graph() {
        let graph = bags_to_contained_by_graph(&to_bags(to_line_results(TEST_RULES)));

        assert_eq!(3, graph.get("faded blue").unwrap().len());
    }

    #[test]
    fn finds_all_potential_containers() {
        let graph = bags_to_contained_by_graph(&to_bags(to_line_results(TEST_RULES)));
        let containers = find_potential_containers("shiny gold", &graph);
        assert_eq!(4, containers.len());

        let empty_containers = find_potential_containers("light red", &graph);
        assert!(empty_containers.is_empty());
    }

    #[test]
    fn converts_bags_to_contains_graph() {
        let bags = to_bags(to_line_results(TEST_RULES));
        let graph = bags_to_contains_graph(&bags);
        let light_red = graph.get("light red").unwrap();
        assert_eq!(2, light_red.len());
    }

    #[test]
    fn finds_correct_bag_count() {
        let bags = to_bags(to_line_results(TEST_RULES));
        let graph = bags_to_contains_graph(&bags);
        let count = find_bag_count("shiny gold", &graph);
        assert_eq!(32, count);

        let alternate_bags = to_bags(to_line_results(ALTERNATE_TEST_RULES));
        let alternate_graph = bags_to_contains_graph(&alternate_bags);
        let alternate_count = find_bag_count("shiny gold", &alternate_graph);
        assert_eq!(126, alternate_count);
    }
}
