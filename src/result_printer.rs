use crate::*;

//the total number of vertical lines ( | ) that appear in the [-|||...|-] in the overview section
static NUM_OF_VERTICALS : usize = 50;

pub fn format_and_print_results(extensions_content_info_ref: &mut ContentInfoRef,
     extensions_metadata_map: &mut HashMap<String, ExtensionMetadata>) 
{
    let mut content_info_map_guard = extensions_content_info_ref.lock();
    let mut content_info_map = content_info_map_guard.as_deref_mut().unwrap();
    
    remove_extensions_with_0_files(&mut content_info_map, extensions_metadata_map);

    let sorted_extension_names = 
        get_extension_names_as_sorted_vec_according_to_how_much_they_appeared(extensions_metadata_map);

    print_individually(&sorted_extension_names, &content_info_map, extensions_metadata_map);

    if extensions_metadata_map.len() > 1 {
        print_overview(&sorted_extension_names, content_info_map, extensions_metadata_map);
    }
}

fn remove_extensions_with_0_files(content_info_map: &mut HashMap<String,ExtensionContentInfo>,
     extensions_metadata_map: &mut HashMap<String, ExtensionMetadata>) 
{
    let mut empty_extensions = Vec::new();
    for element in extensions_metadata_map.iter() {
        if element.1.files == 0 {
            empty_extensions.push(element.0.to_owned());
        }
    }

    for ext in empty_extensions {
        extensions_metadata_map.remove(&ext);
        content_info_map.remove(&ext);
    }
}

fn get_extension_names_as_sorted_vec_according_to_how_much_they_appeared(
    extensions_metadata_map: &HashMap<String, ExtensionMetadata>) -> Vec<String> 
{
    let mut value_map = HashMap::<String,usize>::new();
    let mut sorted_extensions_vec = Vec::new();
    for (ext_name,metadata) in extensions_metadata_map.iter() {
        value_map.insert(ext_name.to_owned(), metadata.files * 10 + metadata.bytes as usize);
        sorted_extensions_vec.push(ext_name.to_owned());
    }

    sorted_extensions_vec.sort_by(|a,b| {
        value_map.get(b).unwrap().cmp(value_map.get(a).unwrap())
    });

    return sorted_extensions_vec
}

fn print_individually(sorted_extensions_map: &[String], content_info_map: &HashMap<String,ExtensionContentInfo>,
     extensions_metadata_map: &HashMap<String, ExtensionMetadata>) 
{
    fn colored_word(word: &str) -> ColoredString {
        word.italic().truecolor(181, 169, 138)
    }

    fn get_size_text(metadata: &ExtensionMetadata) -> String {
        let (size, size_desc) = 
            if metadata.bytes > 1000000 
                {(metadata.bytes as f64 / 1000000f64, colored_word("MBs total"))}
            else if metadata.bytes > 1000 
                {(metadata.bytes as f64 / 1000f64, colored_word("KBs total"))}
            else
                {(metadata.bytes as f64, colored_word("Bytes total"))};

        let (average_size, average_size_desc) =
            if metadata.bytes / metadata.files > 1000 
                {((metadata.bytes as f64 / metadata.files as f64) / 1000f64, colored_word("Kbs average"))}
            else 
                {(metadata.bytes as f64 / metadata.files as f64, colored_word("Bytes average"))};

        format!("{:.1} {} - {:.1} {}",size, size_desc, average_size, average_size_desc)
    }

    println!("{}\n", "Details".underline().bold());
    
    let max_files_num_size = extensions_metadata_map.values().map(|x| x.files).max().unwrap().to_string().len();
    for extension_name in sorted_extensions_map {
        let content_info = content_info_map.get(extension_name).unwrap();
        let metadata = extensions_metadata_map.get(extension_name).unwrap();

        let spaces = get_n_times(" ", 6-extension_name.len());
        let files_str = with_seperators(metadata.files);
        let title = format!(".{}{}{}{} {}  -> ",extension_name.green(), spaces, get_n_times(" ", (max_files_num_size+1)-files_str.len()),
         files_str, colored_word("files"));
        let code_lines_percentage = if content_info.lines > 0 {content_info.code_lines as f64 / content_info.lines as f64 * 100f64} else {0f64};
        let info = format!("{} {} {{{} code ({:.2}%) + {} extra}}  |  {}\n",colored_word("lines"), with_seperators(content_info.lines),
         with_seperators(content_info.code_lines), code_lines_percentage, with_seperators(content_info.lines - content_info.code_lines),
         get_size_text(metadata));
        
        let mut keyword_info = String::new();
        if !content_info.keyword_occurences.is_empty() {
            let mut keyword_iter = content_info.keyword_occurences.iter();
            let first_keyword = keyword_iter.next().unwrap();
            keyword_info.push_str(&format!("{}{}: {}",get_n_times(" ", 19+max_files_num_size), colored_word(first_keyword.0),first_keyword.1));
            for (keyword_name,occurancies) in keyword_iter {
                keyword_info.push_str(&format!(" , {}: {}",colored_word(keyword_name),occurancies));
            }
        }
        println!("{}",format!("{}{}{}\n",title, info, keyword_info));
    }
}

//                                    OVERVIEW
//
// Files:    47% java - 32% cs - 21% py        [-||||||||||||||||||||||||||||||||||||||||||||||||||] 
//
// Lines: ...
//
// Size : ...
fn print_overview(sorted_extension_vec: &[String], content_info_map: &mut HashMap<String, ExtensionContentInfo>,
     extensions_metadata_map: &mut HashMap<String, ExtensionMetadata>) 
{
    fn make_cyan(str: &str) -> String {
        str.bright_cyan().to_string()
    }
    fn make_magenta(str: &str) -> String {
        str.bright_magenta().to_string()
    }
    fn make_yellow(str: &str) -> String {
        str.bright_yellow().to_string()
    }
    fn no_transformation(str: &str) -> String {
        str.to_owned()
    }

    if content_info_map.len() > 3 {
        retain_most_relevant_and_add_others_field_for_rest(sorted_extension_vec, content_info_map, extensions_metadata_map);
    }

    println!("{}\n", "Overview".underline().bold());

    let color_func_vec : Vec<fn(&str) -> String> = vec![make_cyan, make_magenta, make_yellow, no_transformation];

    let files_percentages = get_files_percentages(extensions_metadata_map, sorted_extension_vec);
    let lines_percentages = get_lines_percentages(content_info_map, sorted_extension_vec);
    let sizes_percentages = get_sizes_percentages(extensions_metadata_map, sorted_extension_vec);

    let files_verticals = get_num_of_verticals(&files_percentages);
    let lines_verticals = get_num_of_verticals(&lines_percentages);
    let size_verticals = get_num_of_verticals(&sizes_percentages);

    let files_line = create_overview_line("Files:", &files_percentages, &files_verticals, sorted_extension_vec, &color_func_vec);
    let lines_line = create_overview_line("Lines:", &lines_percentages, &lines_verticals, sorted_extension_vec, &color_func_vec);
    let size_line = create_overview_line("Size :", &sizes_percentages, &size_verticals, sorted_extension_vec, &color_func_vec);

    println!("{}\n\n{}\n\n{}\n",files_line, lines_line, size_line);
}

fn get_num_of_verticals(percentages: &[f64]) -> Vec<usize> {
    let mut verticals = Vec::<usize>::with_capacity(4);
    let mut sum = 0;
    for files_percent in percentages.iter(){
        let num_of_verticals = 
        if *files_percent == 0f64 {
            0
        } else {
            let mut num_of_verticals = (files_percent/2.0).round() as usize;
            if num_of_verticals == 0 {
                num_of_verticals = 1;
            }
            num_of_verticals
        };
        verticals.push(num_of_verticals);
        sum += num_of_verticals;
    }

    if sum != NUM_OF_VERTICALS {
        normalize_to_NUM_OF_VERTICALS(&mut verticals, sum);
    }

    verticals
}

fn normalize_to_NUM_OF_VERTICALS(verticals: &mut Vec<usize>, sum: usize) {
    fn swap_if_needed(sorted_verticals: &mut Vec<&mut usize>, index1: usize, index2: usize) {
        if *sorted_verticals[index2] > *sorted_verticals[index1] {
            let temp = *sorted_verticals[index1];
            *sorted_verticals[index1] = *sorted_verticals[index2];
            *sorted_verticals[index2] = temp;
        }
    }

    let mut sorted_verticals = Vec::new();
    for i in verticals.iter_mut() {
        sorted_verticals.push(i);
    }
    sorted_verticals.sort_by(|a,b| b.cmp(a));

    let is_over = if sum > NUM_OF_VERTICALS {true} else {false};
    let difference = if is_over {sum - NUM_OF_VERTICALS} else {NUM_OF_VERTICALS - sum}; 

    if is_over {
        *sorted_verticals[0] -= 1; 
        swap_if_needed(&mut sorted_verticals, 0, 1);
    } else {
        *sorted_verticals[0] += 1;
    }
    
    for _ in 0..difference-1 {
        if is_over {
            if *sorted_verticals[0] > *sorted_verticals[1] + 3 {
                *sorted_verticals[0] -= 1;
            } else {
                *sorted_verticals[1] -= 1;
                if sorted_verticals.len() > 2 {
                    swap_if_needed(&mut sorted_verticals, 1, 2);
                }
            }
        } else {
            if *sorted_verticals[0] > *sorted_verticals[1] + 5 {
                *sorted_verticals[1] += 1;
                if sorted_verticals.len() > 2 {
                    swap_if_needed(&mut sorted_verticals, 0, 1);
                }
            } else {
                *sorted_verticals[0] += 1;
            }
        }
    }
}

fn create_overview_line(prefix: &str, percentages: &[f64], verticals: &[usize],
        extensions_name: &[String], color_func_vec: &[fn(&str) -> String]) -> String 
{
    let mut line = String::with_capacity(150);
    line.push_str(&format!("{}    ",prefix));
    for (i,percent) in percentages.iter().enumerate() {
        line.push_str(&format!("{:.1}% ",percent)); //make sure that 5 spaces have been accounted for every percentage
        line.push_str(&color_func_vec[i](&extensions_name[i]));
        if i < percentages.len() - 1{
            line.push_str(" - ")
        }
    }
    
    add_verticals_str(&mut line, verticals, color_func_vec);

    line
}

fn add_verticals_str(line: &mut String, files_verticals: &[usize], color_func_vec: &[fn(&str) -> String]) {
    line.push_str("    [-");
    for (i,verticals) in files_verticals.iter().enumerate() {
        line.push_str(&color_func_vec[i]("|").repeat(*verticals));
    }
    line.push_str("-]");
}

fn retain_most_relevant_and_add_others_field_for_rest(sorted_extension_names: &[String],
     content_info_map: &mut HashMap<String, ExtensionContentInfo>, extensions_metadata_map: &mut HashMap<String, ExtensionMetadata>) 
{
    fn name_is_in_top_3(sorted_extension_names: &[String], name: &str)  -> bool {
        name == sorted_extension_names[0] || name == sorted_extension_names[1] || name == sorted_extension_names[2]
    }

    fn get_files_lines_size(content_info_map: &HashMap<String, ExtensionContentInfo>,
         extensions_metadata_map: &HashMap<String, ExtensionMetadata>) -> (usize,usize,usize) 
    {
        let (mut lines, mut files, mut size) = (0,0,0);
        content_info_map.iter().for_each(|x| lines += x.1.lines);
        extensions_metadata_map.iter().for_each(|x| {files += x.1.files; size += x.1.bytes});
        (files, lines, size as usize) 
    }
    
    content_info_map.retain(|x,_| name_is_in_top_3(sorted_extension_names, x));
    extensions_metadata_map.retain(|x,_| name_is_in_top_3(sorted_extension_names, x));
    
    let (total_lines, total_files, total_size) = get_files_lines_size(content_info_map, extensions_metadata_map);
    let (relevant_files, relevant_lines, relevant_size) = get_files_lines_size(content_info_map, extensions_metadata_map);
    let (other_files, other_lines, other_size) = 
        (total_files - relevant_files, total_lines - relevant_lines, total_size - relevant_size);

    content_info_map.insert("other".to_string(), ExtensionContentInfo::dummy(other_lines));
    extensions_metadata_map.insert("other".to_string(), ExtensionMetadata::new(other_files, other_size));
}


fn get_files_percentages(extensions_metadata_map: &HashMap<String,ExtensionMetadata>, sorted_extension_names: &[String]) -> Vec<f64> {
    let mut extensions_files = [0].repeat(extensions_metadata_map.len());
    extensions_metadata_map.iter().for_each(|e| {
        let pos = sorted_extension_names.iter().position(|name| name == e.0).unwrap();
        extensions_files[pos] = e.1.files;
    });
    
    get_percentages(&extensions_files)
}

fn get_lines_percentages(content_info_map: &HashMap<String,ExtensionContentInfo>, extensions_name: &[String]) -> Vec<f64> {
    let mut extensions_lines = [0].repeat(content_info_map.len());
    content_info_map.iter().for_each(|e| {
        let pos = extensions_name.iter().position(|name| name == e.0).unwrap();
        extensions_lines[pos] = e.1.lines;
    });

    get_percentages(&extensions_lines)
}

fn get_sizes_percentages(extensions_metadata_map: &HashMap<String,ExtensionMetadata>, extensions_name: &[String]) -> Vec<f64> {
    let mut extensions_size = [0].repeat(extensions_metadata_map.len());
    extensions_metadata_map.iter().for_each(|e| {
        let pos = extensions_name.iter().position(|name| name == e.0).unwrap();
        extensions_size[pos] = e.1.bytes;
    });
    
    get_percentages(&extensions_size)
}

fn get_percentages(numbers: &[usize]) -> Vec<f64> {
    let total_files :usize = numbers.iter().sum();
    let mut extension_percentages = Vec::with_capacity(4);
    let mut sum = 0.0;
    for (counter,files) in numbers.iter().enumerate() {
        if counter == numbers.len() - 1 {
            extension_percentages.push(100.0 - sum);
        } else {
            let percentage = *files as f64/total_files as f64;
            let percentage = (percentage * 1000f64).round() / 10f64;
            sum += percentage;
            extension_percentages.push(percentage);
        }
    }
    extension_percentages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        let mut verticals = vec![18,15,19,1];
        normalize_to_NUM_OF_VERTICALS(&mut verticals, 53);
        assert_eq!(vec![16,15,18,1], verticals);
        assert!(verticals.iter().sum::<usize>() == 50);
        
        let mut verticals = vec![17,17,18,1];
        normalize_to_NUM_OF_VERTICALS(&mut verticals, 53);
        assert_eq!(vec![16,16,17,1], verticals);
        assert!(verticals.iter().sum::<usize>() == 50);
    
        let mut verticals = vec![16,15,16,1];
        normalize_to_NUM_OF_VERTICALS(&mut verticals, 48);
        assert_eq!(vec![18,15,16,1], verticals);
        assert!(verticals.iter().sum::<usize>() == 50);
    
        let mut verticals = vec![18,16,17];
        normalize_to_NUM_OF_VERTICALS(&mut verticals, 51);
        assert_eq!(vec![17,16,17], verticals);
        assert!(verticals.iter().sum::<usize>() == 50);
    
        let mut verticals = vec![25,26];
        normalize_to_NUM_OF_VERTICALS(&mut verticals, 51);
        assert_eq!(vec![25,25], verticals);
        assert!(verticals.iter().sum::<usize>() == 50);
    }

    #[test]
    fn test_get_lines_percentages() {
        let ext_names = [&"py".to_string(),&"java".to_string(),&"cs".to_string()];

        let content_info_map = hashmap!("cs".to_string() => ExtensionContentInfo::dummy(100),
            "java".to_string() => ExtensionContentInfo::dummy(100), "py".to_string() => ExtensionContentInfo::dummy(0));
        assert_eq!(vec![0f64,50f64,50f64], get_lines_percentages(&content_info_map, &ext_names));

        let content_info_map = hashmap!("cs".to_string() => ExtensionContentInfo::dummy(0),
            "java".to_string() => ExtensionContentInfo::dummy(0), "py".to_string() => ExtensionContentInfo::dummy(1));
        assert_eq!(vec![100f64,0f64,0f64], get_lines_percentages(&content_info_map, &ext_names));
        
        let content_info_map = hashmap!("cs".to_string() => ExtensionContentInfo::dummy(20),
            "java".to_string() => ExtensionContentInfo::dummy(20), "py".to_string() => ExtensionContentInfo::dummy(20));
        assert_eq!(vec![33.33f64,33.33f64,33.34f64], get_lines_percentages(&content_info_map, &ext_names));
    }

    #[test]
    fn test_get_num_of_verticals() {
        let percentages = vec![33.33,33.33,33.34];
        assert_eq!(vec![17,16,17], get_num_of_verticals(&percentages));

        let percentages = vec![0.3,65.67,34.3];
        let verticals = get_num_of_verticals(&percentages);
        assert!(verticals.iter().sum::<usize>() == 50);
        assert_eq!(vec![1,32,17], verticals);
        
        let percentages = vec![0.0,0.0,100.0];
        let verticals = get_num_of_verticals(&percentages);
        assert!(verticals.iter().sum::<usize>() == 50);
        assert_eq!(vec![0,0,50], verticals);
    }
}