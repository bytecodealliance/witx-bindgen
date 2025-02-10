use anyhow::Result;
use std::{fs, path::PathBuf};

use heck::ToUpperCamelCase;

pub struct CSProject;

pub struct CSProjectLLVMBuilder {
    name: String,
    dir: PathBuf,
    aot: bool,
    clean_targets: bool,
    world_name: String,
}

pub struct CSProjectMonoBuilder {
    name: String,
    dir: PathBuf,
    aot: bool,
    clean_targets: bool,
    world_name: String,
}

impl CSProject {
    pub fn new(dir: PathBuf, name: &str, world_name: &str) -> CSProjectLLVMBuilder {
        CSProjectLLVMBuilder {
            name: name.to_string(),
            dir,
            aot: false,
            clean_targets: false,
            world_name: world_name.to_string(),
        }
    }

    pub fn new_mono(dir: PathBuf, name: &str, world_name: &str) -> CSProjectMonoBuilder {
        CSProjectMonoBuilder {
            name: name.to_string(),
            dir,
            aot: false,
            clean_targets: false,
            world_name: world_name.to_string(),
        }
    }
}

impl CSProjectLLVMBuilder {
    pub fn generate(&self) -> Result<()> {
        let name = &self.name;
        let world = &self.world_name.replace("-", "_");
        let camel = format!("{}World", world.to_upper_camel_case());

        fs::write(
            self.dir.join("rd.xml"),
            format!(
                r#"<Directives xmlns="http://schemas.microsoft.com/netfx/2013/01/metadata">
            <Application>
                <Assembly Name="{name}">
                </Assembly>
            </Application>
        </Directives>"#
            ),
        )?;

        let mut csproj = format!(
            "<Project Sdk=\"Microsoft.NET.Sdk\">
    
        <PropertyGroup>
            <TargetFramework>net9.0</TargetFramework>
            <LangVersion>preview</LangVersion>
            <RootNamespace>{name}</RootNamespace>
            <ImplicitUsings>enable</ImplicitUsings>
            <Nullable>enable</Nullable>
            <AllowUnsafeBlocks>true</AllowUnsafeBlocks>
            <!-- treat these are errors so they are caught during code generation tests -->
            <WarningsAsErrors>CS0105</WarningsAsErrors>
        </PropertyGroup>
        
        <PropertyGroup>
            <PublishTrimmed>true</PublishTrimmed>
            <AssemblyName>{name}</AssemblyName>
        </PropertyGroup>

        <ItemGroup>
            <RdXmlFile Include=\"rd.xml\" />
        </ItemGroup>

        <ItemGroup>
            <CustomLinkerArg Include=\"-Wl,--component-type,{camel}_component_type.wit\" />
        </ItemGroup>
        "
        );

        if self.aot {
            csproj.push_str(
                r#"
                <ItemGroup>
                    <PackageReference Include="Microsoft.DotNet.ILCompiler.LLVM" Version="10.0.0-*" />
                    <PackageReference Include="runtime.win-x64.Microsoft.DotNet.ILCompiler.LLVM" Version="10.0.0-*" />
                </ItemGroup>
                "#,
            );

            fs::write(
                self.dir.join("nuget.config"),
                r#"<?xml version="1.0" encoding="utf-8"?>
            <configuration>
                <config>
                    <!-- Store the packages where they can be shared between tests -->
                    <add key="globalPackagesFolder" value="../../.packages" />
                </config>
                <packageSources>
                <!--To inherit the global NuGet package sources remove the <clear/> line below -->
                <clear />
                <add key="nuget" value="https://api.nuget.org/v3/index.json" />
                <add key="dotnet-experimental" value="https://pkgs.dev.azure.com/dnceng/public/_packaging/dotnet-experimental/nuget/v3/index.json" />
                <!--<add key="dotnet-experimental" value="C:\github\runtimelab\artifacts\packages\Debug\Shipping" />-->
              </packageSources>
            </configuration>"#,
            )?;
        }

        if self.clean_targets {
            let mut wasm_filename = self.dir.join(name);
            wasm_filename.set_extension("wasm");
            // In CI we run out of disk space if we don't clean up the files, we don't need to keep any of it around.
            csproj.push_str(&format!(
                "<Target Name=\"CleanAndDelete\"  AfterTargets=\"Clean\">
                <!-- Remove obj folder -->
                <RemoveDir Directories=\"$(BaseIntermediateOutputPath)\" />
                <!-- Remove bin folder -->
                <RemoveDir Directories=\"$(BaseOutputPath)\" />
                <RemoveDir Directories=\"{}\" />
                <RemoveDir Directories=\".packages\" />
            </Target>",
                wasm_filename.display()
            ));
        }

        csproj.push_str(
            r#"</Project>
            "#,
        );

        fs::write(self.dir.join(format!("{camel}.csproj")), csproj)?;

        Ok(())
    }

    pub fn aot(&mut self) {
        self.aot = true;
    }

    pub fn clean(&mut self) -> &mut Self {
        self.clean_targets = true;

        self
    }
}

impl CSProjectMonoBuilder {
    pub fn generate(&self) -> Result<()> {
        let name = &self.name;
        let world = &self.world_name.replace("-", "_");
        let camel = format!("{}World", world.to_upper_camel_case());

        let aot = self.aot;

        let maybe_aot = match aot {
            true => format!("<WasmBuildNative>{aot}</WasmBuildNative>"),
            false => String::new(),
        };

        let mut csproj = format!(
            "<Project Sdk=\"Microsoft.NET.Sdk\">
    
        <PropertyGroup>
            <TargetFramework>net9.0</TargetFramework>
            <RuntimeIdentifier>wasi-wasm</RuntimeIdentifier>
            <OutputType>Library</OutputType>
            {maybe_aot}
            <RunAOTCompilation>{aot}</RunAOTCompilation>
            <WasmNativeStrip>false</WasmNativeStrip>
            <WasmSingleFileBundle>true</WasmSingleFileBundle>
            <RootNamespace>{name}</RootNamespace>
            <ImplicitUsings>enable</ImplicitUsings>
            <Nullable>enable</Nullable>
            <AllowUnsafeBlocks>true</AllowUnsafeBlocks>
            <!-- treat these are errors so they are caught during code generation tests -->
            <WarningsAsErrors>CS0105</WarningsAsErrors>
        </PropertyGroup>
        
        <PropertyGroup>
            <PublishTrimmed>true</PublishTrimmed>
            <AssemblyName>{name}</AssemblyName>
        </PropertyGroup>

        <ItemGroup>
          <NativeFileReference Include=\"{camel}_component_type.o\" Condition=\"Exists('{camel}_component_type.o')\"/>
        </ItemGroup>

        "
        );

        fs::write(
            self.dir.join("nuget.config"),
            r#"<?xml version="1.0" encoding="utf-8"?>
        <configuration>
            <config>
                <add key="globalPackagesFolder" value=".packages" />
            </config>
            <packageSources>
                <!--To inherit the global NuGet package sources remove the <clear/> line below -->
                <clear />
                <add key="nuget" value="https://api.nuget.org/v3/index.json" />
                <add key="dotnet9" value="https://pkgs.dev.azure.com/dnceng/public/_packaging/dotnet9/nuget/v3/index.json" />
            </packageSources>
        </configuration>"#,
        )?;

        if self.clean_targets {
            let mut wasm_filename = self.dir.join(name);
            wasm_filename.set_extension("wasm");
            // In CI we run out of disk space if we don't clean up the files, we don't need to keep any of it around.
            csproj.push_str(&format!(
                "<Target Name=\"CleanAndDelete\"  AfterTargets=\"Clean\">
                <!-- Remove obj folder -->
                <RemoveDir Directories=\"$(BaseIntermediateOutputPath)\" />
                <!-- Remove bin folder -->
                <RemoveDir Directories=\"$(BaseOutputPath)\" />
                <RemoveDir Directories=\"{}\" />
                <RemoveDir Directories=\".packages\" />
            </Target>",
                wasm_filename.display()
            ));
        }

        csproj.push_str(
            r#"</Project>
            "#,
        );

        fs::write(self.dir.join(format!("{camel}.csproj")), csproj)?;

        Ok(())
    }

    pub fn aot(&mut self) {
        self.aot = true;
    }

    pub fn clean(&mut self) -> &mut Self {
        self.clean_targets = true;

        self
    }
}
