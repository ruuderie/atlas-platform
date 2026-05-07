import re

with open('apps/anchor/src/pages/admin.rs', 'r') as f:
    lines = f.readlines()

new_lines = []
i = 0
while i < len(lines):
    line = lines[i]
    if "<Transition fallback" in line:
        # Check if the next line is a table
        j = i + 1
        while j < len(lines) and lines[j].strip() == "":
            j += 1
        
        if "<table " in lines[j]:
            # Found a Transition wrapping a table
            new_lines.append(line)
            # Find the resource being matched
            # Usually it's `{move || match some_resource.get() {` inside the tbody
            k = j
            resource_name = ""
            while k < len(lines) and "collect_view" not in lines[k]:
                if "{move || match " in lines[k]:
                    m = re.search(r'match (.*?)\.get\(\)', lines[k])
                    if m:
                        resource_name = m.group(1)
                elif "{move || " in lines[k]:
                    m = re.search(r'let.*?=\s*(.*?)\.get\(\)', lines[k])
                    if m:
                        resource_name = m.group(1)
                k += 1
            
            # If we didn't find the resource directly, let's just use a dummy read
            # Actually, we can just do {move || { view! { ... } }} without reading the resource early!
            # If we just wrap the table in {move || { view! { ... } }}, Leptos will detect the resource read inside the table's children!
            # Yes! Leptos suspense catches ANY resource read inside the tree created by the closure!
            # So we literally just need to insert `{move || view! {` before the table, and `}.into_view()}` after the table!
            
            new_lines.append("            {move || view! {\n")
            
            # Now copy lines until </table>
            while j < len(lines):
                new_lines.append(lines[j])
                if "</table>" in lines[j]:
                    break
                j += 1
                
            new_lines.append("            }.into_view()}\n")
            
            # Then the next line should be </Transition>
            j += 1
            while j < len(lines) and lines[j].strip() == "":
                j += 1
            
            if "</Transition>" in lines[j]:
                new_lines.append(lines[j])
            else:
                new_lines.append("        </Transition>\n")
                new_lines.append(lines[j]) # whatever else
                
            i = j + 1
            continue
            
    new_lines.append(line)
    i += 1

with open('apps/anchor/src/pages/admin.rs', 'w') as f:
    f.writelines(new_lines)

