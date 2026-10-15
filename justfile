slang file:
    slangc -target spirv -fvk-use-entrypoint-name -emit-spirv-directly -capability spvDescriptorHeapEXT {{file}}.slang -o {{file}}.spv
