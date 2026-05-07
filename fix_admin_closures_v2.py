import re

def fix_admin():
    with open('apps/anchor/src/pages/admin.rs', 'r') as f:
        content = f.read()

    pattern = r'(\s*)\{move \|\| view! \{\n(.*?)\s*\{move \|\| match ([a-zA-Z0-9_]+)\.get\(\) \{(.*?)\}\.into_view()\}'
    
    def repl(m):
        indent = m.group(1)
        between = m.group(2)
        resource = m.group(3)
        rest = m.group(4)
        
        res = f"{indent}{{move || {{\n{indent}    let res = {resource}.get();\n{indent}    view! {{\n{between}{indent}        {{match res {{{rest}}}.into_view()\n{indent}}}}}"
        return res

    new_content = re.sub(pattern, repl, content, flags=re.DOTALL)
    
    with open('apps/anchor/src/pages/admin.rs', 'w') as f:
        f.write(new_content)

def fix_admin_modal():
    with open('apps/anchor/src/components/admin_modal.rs', 'r') as f:
        content = f.read()

    pattern = r'(<Transition fallback=[^>]+>)\n(\s*)(<div[^>]*>)\n\s*\{move \|\| match ([a-zA-Z0-9_]+)\.get\(\) \{(.*?)\}\}\n(\s*)(</div>)\n\s*(</Transition>)'
    
    def repl(m):
        transition_tag = m.group(1)
        indent = m.group(2)
        div_tag = m.group(3)
        resource = m.group(4)
        inner = m.group(5)
        close_indent = m.group(6)
        div_close = m.group(7)
        transition_close = m.group(8)
        
        # fix braces in f-string
        res = (
            f"{transition_tag}\n"
            f"{indent}{{move || {{\n"
            f"{indent}    let res = {resource}.get();\n"
            f"{indent}    view! {{\n"
            f"{indent}    {div_tag}\n"
            f"{indent}        {{match res {{{inner}}}}}\n"
            f"{indent}    {div_close}\n"
            f"{indent}    }}.into_view()\n"
            f"{indent}}}}}\n"
            f"{indent}{transition_close}"
        )
        return res

    new_content = re.sub(pattern, repl, content, flags=re.DOTALL)
    
    with open('apps/anchor/src/components/admin_modal.rs', 'w') as f:
        f.write(new_content)

fix_admin()
fix_admin_modal()

