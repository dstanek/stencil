import re
from pathlib import Path
from subprocess import run, CompletedProcess

import pytest

DEST = "/app/dest"


class Result:
    def __init__(self, res: CompletedProcess):
        self.res = res

    def returncode(self, code: int):
        assert self.res.returncode == code
        return self


    def strip_color_codes(self, text: str) -> str:
        return re.sub(r"\x1b\[[0-9;]*m", "", text)
    
    def stdout_contains(self, text: str):
        assert text in self.strip_color_codes(self.res.stdout)
        return self
    
    def not_stdout_contains(self, text: str):
        assert text not in self.strip_color_codes(self.res.stdout)
        return self
    
    def stderr_contains(self, text: str):
        assert text in self.strip_color_codes(self.res.stderr)
        return self


class Stencil:
    def __init__(self, cmd: str, *args: str, dest: Path, **kwargs: str):
        self.cmd = cmd
        self.args = args
        self.dest = dest
        self.kwargs = kwargs
        self.result = self.run()


    def run(self) -> Result:
        cmd = ["/stencil/stencil", self.cmd, *self.args]
        for name, value in self.kwargs.items():
            if len(name) == 1:
                cmd.append(f"-{name}")
            else:
                cmd.append(f"--{name}")
            cmd.append(value)
        return Result(run(cmd, capture_output=True, text=True))


def slurp(directory: Path) -> dict[str, str]:
    print("Directory:", directory)
    rv = {}
    for path in directory.rglob("*"):
        print(path)
        if path.is_file():
            rv[str(path.relative_to(directory))] = path.read_text()
    return rv


def describe_stencil_init():

    def describe_init_on_existing_directory():

        @pytest.fixture(scope="function")
        def stencil(tmp_path):
            yield Stencil("init", str(tmp_path), "./stencil1_src", dest=tmp_path)

        def it_fails_with_message(stencil):
            assert Path(stencil.args[0]).exists()
            (stencil.result
                .returncode(1)
                .stderr_contains(f"Error: Destination '{stencil.args[0]}' already exists"))


def describe_stencil_lifecycle():

    STENCIL_PATH = "./stencil1_src/stencil"
    STENCIL_OVERRIDE_PATH = "./stencil2_src/stencil"

    @pytest.fixture(scope="module")
    def stencil_init(tmp_path_factory):
        tmp_path = tmp_path_factory.mktemp("output") / "output"
        yield Stencil("init", str(tmp_path), STENCIL_PATH, dest=tmp_path)

    def describe_successful_init():

        def it_returns_success(stencil_init):
            (stencil_init.result
                .returncode(0)
                .stdout_contains(f"Successfully initialized {stencil_init.dest}"))

        def it_creates_directory(stencil_init):
            assert Path(stencil_init.dest).exists()
    
        def it_creates_files(stencil_init):
            files = slurp(Path(stencil_init.dest))
            expected_files = {
                ".stencil.toml": f'[stencil]\nversion = "1"\n\n[project]\nname = "my_project"\nsrc = "{STENCIL_PATH}"\n',
                "README.md": "# my_project Documentation\n\nA\nB\nC",
                "pyproject.toml": "[project]\nname = my_project",
                ".github/CODEOWNERS": "* @all_the_engineers",
                "my_project/__init__.py": "",
            }
            assert files == expected_files

        def it_contains_new_directories_in_the_diff(stencil_init):
            stencil_init.result.stdout_contains("--- old/my_project    (directory not found)")
            stencil_init.result.stdout_contains("+++ new/my_project    (new directory)")

        def it_contains_new_empty_file_in_the_diff(stencil_init):
            stencil_init.result.stdout_contains("--- old/my_project/__init__.py    (file not found)")
            stencil_init.result.stdout_contains("+++ new/my_project/__init__.py    (new empty file)")

        def it_contains_new_file_in_the_diff(stencil_init):
            stencil_init.result.stdout_contains("--- old/pyproject.toml    (file not found)")
            stencil_init.result.stdout_contains("+++ new/pyproject.toml")

    def describe_checking_for_changes():

        @pytest.fixture(scope="module")
        def stencil_plan(stencil_init):
            yield Stencil(
                 "--override", f"project.src={STENCIL_OVERRIDE_PATH}",
                "plan",
                 str(stencil_init.dest),
                 dest=stencil_init.dest,
            )

        def it_returns_success(stencil_plan):
            (stencil_plan.result
                .returncode(0)
                .stdout_contains(f"Planning {stencil_plan.dest} changes"))

        def it_contains_new_file_in_the_diff(stencil_plan):
            stencil_plan.result.stdout_contains("--- old/my_project/__version__.py    (file not found)")
            stencil_plan.result.stdout_contains("+++ new/my_project/__version__.py")

        def it_does_not_contain_unchanged_files_in_the_diff(stencil_plan):
            stencil_plan.result.not_stdout_contains("pyproject.toml")

        def it_contains_file_updates_in_the_diff(stencil_plan):
            stencil_plan.result.stdout_contains("--- old/README.md")
            stencil_plan.result.stdout_contains("+++ new/README.md")
            stencil_plan.result.stdout_contains("-   4 B")
            stencil_plan.result.stdout_contains("+   4 X")

        def it_contains_new_directories_in_the_diff(stencil_plan):
            stencil_plan.result.stdout_contains("--- old/tests    (directory not found)")
            stencil_plan.result.stdout_contains("+++ new/tests    (new directory)")

    def describe_applying_changes():

        @pytest.fixture(scope="module")
        def stencil_apply(stencil_init):
            yield Stencil(
                 "--override", f"project.src={STENCIL_OVERRIDE_PATH}",
                "apply",
                 str(stencil_init.dest),
                 dest=stencil_init.dest,
            )

        def it_returns_success(stencil_apply):
            (stencil_apply.result
                .returncode(0)
                .stdout_contains(f"Applying changes from {STENCIL_OVERRIDE_PATH} to {stencil_apply.dest}"))

#                def it_creates_directory(stencil):
#                    assert Path(stencil.args[0]).exists()
#        
#                def it_creates_files(stencil):
#                    files = slurp(Path(stencil.args[0]))
#                    expected_files = {
#                        '.stencil.toml': '[stencil]\nversion = "1"\n\n[project]\nname = "my_project"\nsrc = "./stencil1_src"\n',
#                        'stencil/README.md': '# my_project Documentation',
#                        'stencil/pyproject.toml': '[project]\nname = my_project',
#                        'stencil/.github/CODEOWNERS': '* @all_the_engineers',
#                        'stencil/my_project/__init__.py': ''
#

# ".github/workflow.yaml": "name: CI for my_project\non:\n  push:\n    branches:\n      - main"

#                    }
#                    assert files == expected_files

        def it_contains_new_file_in_the_diff(stencil_apply):
            stencil_apply.result.stdout_contains("--- old/my_project/__version__.py    (file not found)")
            stencil_apply.result.stdout_contains("+++ new/my_project/__version__.py")

        def it_does_not_contain_unchanged_files_in_the_diff(stencil_apply):
            stencil_apply.result.not_stdout_contains("pyproject.toml")

        def it_contains_file_updates_in_the_diff(stencil_apply):
            stencil_apply.result.stdout_contains("--- old/README.md")
            stencil_apply.result.stdout_contains("+++ new/README.md")
            stencil_apply.result.stdout_contains("-   4 B")
            stencil_apply.result.stdout_contains("+   4 X")

        def it_contains_new_directories_in_the_diff(stencil_apply):
            stencil_apply.result.stdout_contains("--- old/tests    (directory not found)")
            stencil_apply.result.stdout_contains("+++ new/tests    (new directory)")

        def it_creates_and_updates_files(stencil_init):
            files = slurp(Path(stencil_init.dest))
            expected_files = {
                ".stencil.toml": f'[stencil]\nversion = "1"\n\n[project]\nname = "my_project"\nsrc = "{STENCIL_PATH}"\n',
                "README.md": "# my_project Documentation\n\nA\nX\nC",
                "pyproject.toml": "[project]\nname = my_project",
                ".github/CODEOWNERS": "* @all_the_engineers\n* @all_the_managers",
                "my_project/__init__.py": "",
                "my_project/__version__.py": '__version__ = "TODO: your version here"',
                "tests/__init__.py": "",
                "tests/test_example.py": "# some example tests\ndef test():\n    assert True"
            }
            assert files == expected_files
#
#                def describe_init_on_existing_directory():
#
#                    @pytest.fixture(scope="function")
#                    def stencil(tmp_path):
#                        yield Stencil("init", str(tmp_path / "output"), "./stencil1_src")
#
#                    def it_fails_with_message(stencil):
#                        assert Path(stencil.args[0]).exists()
#                        (stencil.result
#                            .returncode(1)
#                            .stderr_contains(f"Error: Destination '{stencil.args[0]}' already exists"))
#
                #def describe_successful_init():
#
#                    @pytest.fixture(scope="function")
#                    def stencil(tmp_path):
#                        yield Stencil("init", str(tmp_path / "output"), "./stencil1_src")
#                
#                    def it_returns_success(stencil):
#                        (stencil.result
#                            .returncode(0)
#                            .stdout_contains(f"Successfully initialized {stencil.args[0]}"))
#
#                    def it_creates_directory(stencil):
#                        assert Path(stencil.args[0]).exists()
#                
#                    def it_creates_files(stencil):
#                        files = slurp(Path(stencil.args[0]))
#                        expected_files = {
#                            '.stencil.toml': '[stencil]\nversion = "1"\n\n[project]\nname = "my_project"\nsrc = "./stencil1_src"\n',
#                            'stencil/README.md': '# my_project Documentation',
#                            'stencil/pyproject.toml': '[project]\nname = my_project',
#                            'stencil/.github/CODEOWNERS': '* @
#    

def describe_github_stencil():

    STENCIL_PATH = "gh://dstanek/stencil-test"

    @pytest.fixture(scope="module")
    def stencil_init(tmp_path_factory):
        tmp_path = tmp_path_factory.mktemp("output") / "output"
        yield Stencil("init", str(tmp_path), STENCIL_PATH, dest=tmp_path)

    def describe_successful_init():

        def it_returns_success(stencil_init):
            (stencil_init.result
                .returncode(0)
                .stdout_contains(f"Successfully initialized {stencil_init.dest}"))

        def it_creates_directory(stencil_init):
            assert Path(stencil_init.dest).exists()
    
        def it_creates_files(stencil_init):
            files = slurp(Path(stencil_init.dest))
            expected_files = {
                ".github/CODEOWNERS": "* @all_the_engineers\n* @all_the_managers",
                ".stencil.toml": f'[stencil]\nversion = "1"\n\n[project]\nname = "my_project"\nsrc = "{STENCIL_PATH}"\n',
                "README.md": "# my_project Documentation\n\nX\nY\nZ\n",
                "pyproject.toml": "[project]\nname = my_project",
                "my_project/__init__.py": "",
                "my_project/__version__.py": "__version__ = \"TODO: your version here\"",
                "tests/__init__.py": "",
                "tests/test_example.py": "# some example tests\ndef test():\n    assert True"
            }
            assert files == expected_files

        def it_contains_new_directories_in_the_diff(stencil_init):
            stencil_init.result.stdout_contains("--- old/my_project    (directory not found)")
            stencil_init.result.stdout_contains("+++ new/my_project    (new directory)")

        def it_contains_new_empty_file_in_the_diff(stencil_init):
            stencil_init.result.stdout_contains("--- old/my_project/__init__.py    (file not found)")
            stencil_init.result.stdout_contains("+++ new/my_project/__init__.py    (new empty file)")

        def it_contains_new_file_in_the_diff(stencil_init):
            stencil_init.result.stdout_contains("--- old/pyproject.toml    (file not found)")
            stencil_init.result.stdout_contains("+++ new/pyproject.toml")


def describe_github_stencil_using_alternative_path():

    STENCIL_PATH = "gh://dstanek/stencil-test/other_stencil"

    @pytest.fixture(scope="module")
    def stencil_init(tmp_path_factory):
        tmp_path = tmp_path_factory.mktemp("output") / "output"
        yield Stencil("init", str(tmp_path), STENCIL_PATH, dest=tmp_path)

    def describe_successful_init():

        def it_returns_success(stencil_init):
            (stencil_init.result
                .returncode(0)
                .stdout_contains(f"Successfully initialized {stencil_init.dest}"))

        def it_creates_directory(stencil_init):
            assert Path(stencil_init.dest).exists()
    
        def it_creates_files(stencil_init):
            files = slurp(Path(stencil_init.dest))
            expected_files = {
                ".github/CODEOWNERS": "* @all_the_engineers\n* @all_the_managers",
                ".stencil.toml": f'[stencil]\nversion = "1"\n\n[project]\nname = "my_project"\nsrc = "{STENCIL_PATH}"\n',
                "README.md": "# my_project Other Documentation\n\nA\nB\nC\n",
                "pyproject.toml": "[project]\nname = my_project",
                "my_project-other/__init__.py": "",
                "my_project-other/__version__.py": "__version__ = \"TODO: your version here\"",
                "tests/__init__.py": "",
                "tests/test_example.py": "# some example tests\ndef test():\n    assert True"
            }
            assert files == expected_files

        def it_contains_new_directories_in_the_diff(stencil_init):
            stencil_init.result.stdout_contains("--- old/my_project-other    (directory not found)")
            stencil_init.result.stdout_contains("+++ new/my_project-other    (new directory)")

        def it_contains_new_empty_file_in_the_diff(stencil_init):
            stencil_init.result.stdout_contains("--- old/my_project-other/__init__.py    (file not found)")
            stencil_init.result.stdout_contains("+++ new/my_project-other/__init__.py    (new empty file)")

        def it_contains_new_file_in_the_diff(stencil_init):
            stencil_init.result.stdout_contains("--- old/pyproject.toml    (file not found)")
            stencil_init.result.stdout_contains("+++ new/pyproject.toml")

# Usecases:
  # directory changes show up as new directories
  # stencil does not keep track of things it needs to delete!

# def describe_github_stencil_using_incorrect_path():
#     pass