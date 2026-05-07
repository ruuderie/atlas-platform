import re
with open('apps/anchor/src/pages/admin.rs', 'r') as f:
    content = f.read()

pattern = r'(\s*)\{move \|\| view! \{\n(.*?)\s*\{move \|\| match ([a-zA-Z0-9_]+)\.get\(\) \{(.*?)\}\.into_view()\}'
# The `rest` group needs to capture everything up to `            }.into_view()}`.
# Wait, let's just replace `\{move \|\| match` with `{match` and remove `move || view! {` and `}.into_view()}` using string replacement.

lines = content.split('\n')
new_lines = []
i = 0
while i < len(lines):
    if lines[i].strip() == "{move || view! {":
        indent = lines[i][:len(lines[i]) - len(lines[i].lstrip())]
        
        # find inner match
        inner_idx = -1
        resource = ""
        for j in range(i+1, min(i+50, len(lines))):
            m = re.search(r'\{move \|\| match ([a-zA-Z0-9_]+)\.get\(\) \{', lines[j])
            if m:
                inner_idx = j
                resource = m.group(1)
                break
                
        if inner_idx != -1:
            new_lines.append(f"{indent}{{move || {{")
            new_lines.append(f"{indent}    let res = {resource}.get();")
            new_lines.append(f"{indent}    view! {{")
            
            i += 1
            while i < inner_idx:
                new_lines.append(lines[i])
                i += 1
                
            # inner line
            new_lines.append(lines[i].replace(f"{{move || match {resource}.get() {{", "{match res {"))
            
            i += 1
            while i < len(lines):
                if lines[i].strip() == "}.into_view()}":
                    new_lines.append(lines[i].replace("}.into_view()}", "}.into_view()\n" + indent + "}"))
                    break
                else:
                    new_lines.append(lines[i])
                i += 1
            i += 1
            continue
    new_lines.append(lines[i])
    i += 1

with open('apps/anchor/src/pages/admin.rs', 'w') as f:
    f.write('\n'.join(new_lines))

