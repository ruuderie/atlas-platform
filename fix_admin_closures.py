import re

def fix_admin():
    with open('apps/anchor/src/pages/admin.rs', 'r') as f:
        content = f.read()

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
                        new_lines.append(lines[i].replace("}.into_view()}", "}.into_view()\n" + indent + "}}"))
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

def fix_admin_modal():
    with open('apps/anchor/src/components/admin_modal.rs', 'r') as f:
        lines = f.read().split('\n')

    new_lines = []
    i = 0
    while i < len(lines):
        line = lines[i]
        if "<Transition fallback" in line:
            indent = line[:len(line) - len(line.lstrip())]
            
            inner_idx = -1
            resource = ""
            for j in range(i+1, min(i+10, len(lines))):
                m = re.search(r'\{move \|\| match ([a-zA-Z0-9_]+)\.get\(\) \{', lines[j])
                if m:
                    inner_idx = j
                    resource = m.group(1)
                    break
                    
            if inner_idx != -1:
                new_lines.append(line)
                new_lines.append(f"{indent}    {{move || {{")
                new_lines.append(f"{indent}        let res = {resource}.get();")
                new_lines.append(f"{indent}        view! {{")
                
                i += 1
                while i < inner_idx:
                    new_lines.append("    " + lines[i])
                    i += 1
                    
                new_lines.append("    " + lines[i].replace(f"{{move || match {resource}.get() {{", "{match res {"))
                
                i += 1
                while i < len(lines):
                    if "</Transition>" in lines[i]:
                        new_lines.append(f"{indent}        }}.into_view()")
                        new_lines.append(f"{indent}    }}}}")
                        new_lines.append(lines[i])
                        break
                    else:
                        new_lines.append("    " + lines[i])
                    i += 1
                i += 1
                continue
                
        new_lines.append(line)
        i += 1

    with open('apps/anchor/src/components/admin_modal.rs', 'w') as f:
        f.write('\n'.join(new_lines))

fix_admin()
fix_admin_modal()

