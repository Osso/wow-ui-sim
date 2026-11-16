//! Supplemental registration inventory for behavior-neutral before/after proof.
#![cfg(feature = "optional-tests")]

#[test]
fn dump_registered_c_namespaces() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    let dump: String = env.eval(r#"
            local lines = {}
            local function dump(scope, root)
                if type(root) ~= 'table' then return end
                for name, namespace in next, root do
                    if type(name)=='string' and name:match('^C_') and type(namespace)=='table' then
                        table.insert(lines, scope..'\t'..name..'\t<namespace>')
                        for key, value in next, namespace do
                            assert(type(key)=='string' or type(key)=='number')
                            table.insert(lines, scope..'\t'..name..'\t'..tostring(key)..'\t'..type(value))
                        end
                    end
                end
            end
            dump('ordinary',_G)
            dump('secure',rawget(_G,'__secureenv'))
            table.sort(lines)
            return table.concat(lines,'\n')
        "#).unwrap();
    println!("LT7869_NAMESPACE_BEGIN\n{dump}\nLT7869_NAMESPACE_END");
}
