# Copyright (c) 2024-2025 David Stanek <dstanek@dstanek.com>

from pathlib import Path
from textwrap import dedent

import pytest

from lib import Stencil, slurp

DEST = "/app/dest"


def describe_stencil_init():

    def describe_when_target_directory_exists():

        @pytest.fixture(scope="module")
        def stencil(tmp_path_factory):
            tmp_path = tmp_path_factory.mktemp("output")
            yield (Stencil.init()
                .dest(tmp_path)
                .src("./stencil1_src")
                .run())

        def it_fails_with_message(stencil):
            assert Path(stencil.dest).exists()
            (stencil
                .returncode(1)
                .stderr_contains(f"Error: Destination '{stencil.dest}' already exists"))

    # TODO: def describe_when_target_directory_is_unwritalble():

    def describe_when_providing_arguments():

        STENCIL_PATH = "./complete-stencil/stencil"
        PROJECT_NAME = "my_project"

        @pytest.fixture(scope="module")
        def stencil_init(tmp_path_factory):
            tmp_path = tmp_path_factory.mktemp("output") / "output"
            yield (Stencil.init()
                .arg("-a", "arg0=0")
                .arg("--argument", "arg1=one")
                .arg("-a", "arg2=\"zero one two\"")
                .dest(tmp_path)
                .src(STENCIL_PATH)
                .run())

        def it_succeeds(stencil_init):
            (stencil_init
                .returncode(0)
                .stdout_contains(f"Successfully initialized {stencil_init.dest}"))

        def it_creates_files(stencil_init):
            files = slurp(stencil_init.dest)
            expected_files = {
                ".stencil.toml": dedent(f"""\
                    [stencil]
                    version = "1"

                    [project]
                    name = "{PROJECT_NAME}"
                    src = "{STENCIL_PATH}"

                    [arguments]
                    arg0 = "0"
                    arg1 = "one"
                    arg2 = "zero one two"
                """),
                "README.md": f"# {PROJECT_NAME} Documentation\n\nA\nB\nC\n",
                "pyproject.toml": f"[project]\nname = {PROJECT_NAME}\n",
                ".github/CODEOWNERS": "* @all_the_engineers\n",
                "my_project/__init__.py": dedent("""\
                    # show the arguments here
                    arg0 = 0
                    arg1 = "one"
                    arg2 = "zero one two"
                """),
            }
            assert files == expected_files


def describe_stencil_lifecycle():

    STENCIL_PATH = "./stencil1_src/stencil"
    STENCIL_OVERRIDE_PATH = "./stencil2_src/stencil"

    @pytest.fixture(scope="module")
    def stencil_init(tmp_path_factory):
        tmp_path = tmp_path_factory.mktemp("output") / "output"
        yield (Stencil.init()
               .dest(tmp_path)
               .src(STENCIL_PATH)
               .run())

    def describe_successful_init():

        def it_returns_success(stencil_init):
            (stencil_init
                .returncode(0)
                .stdout_contains(f"Successfully initialized {stencil_init.dest}"))

        def it_creates_directory(stencil_init):
            assert stencil_init.dest.exists()

        def it_creates_files(stencil_init):
            files = slurp(stencil_init.dest)
            expected_files = {
                ".stencil.toml": f'[stencil]\nversion = "1"\n\n[project]\nname = "my_project"\nsrc = "{STENCIL_PATH}"\n\n[arguments]\n',
                "README.md": "# my_project Documentation\n\nA\nB\nC\n",
                "pyproject.toml": "[project]\nname = my_project\n",
                ".github/CODEOWNERS": "* @all_the_engineers\n",
                "my_project/__init__.py": "",
            }
            assert files == expected_files

        def it_contains_new_directories_in_the_diff(stencil_init):
            stencil_init.stdout_contains("--- old/my_project    (directory not found)")
            stencil_init.stdout_contains("+++ new/my_project    (new directory)")

        def it_contains_new_empty_file_in_the_diff(stencil_init):
            stencil_init.stdout_contains("--- old/my_project/__init__.py    (file not found)")
            stencil_init.stdout_contains("+++ new/my_project/__init__.py    (new empty file)")

        def it_contains_new_file_in_the_diff(stencil_init):
            stencil_init.stdout_contains("--- old/pyproject.toml    (file not found)")
            stencil_init.stdout_contains("+++ new/pyproject.toml")

    def describe_checking_for_changes():

        @pytest.fixture(scope="module")
        def stencil_plan(stencil_init):
            yield (Stencil.plan()
                .dest(stencil_init.dest)
                .override("project.src", STENCIL_OVERRIDE_PATH)
                .run())

        def it_returns_success(stencil_plan):
            (stencil_plan
                .returncode(0)
                .stdout_contains(f"Planning {stencil_plan.dest} changes"))

        def it_contains_new_file_in_the_diff(stencil_plan):
            stencil_plan.stdout_contains("--- old/my_project/__version__.py    (file not found)")
            stencil_plan.stdout_contains("+++ new/my_project/__version__.py")

        def it_does_not_contain_unchanged_files_in_the_diff(stencil_plan):
            stencil_plan.not_stdout_contains("pyproject.toml")

        def it_contains_file_updates_in_the_diff(stencil_plan):
            stencil_plan.stdout_contains("--- old/README.md")
            stencil_plan.stdout_contains("+++ new/README.md")
            stencil_plan.stdout_contains("-   4 B")
            stencil_plan.stdout_contains("+   4 X")

        def it_contains_new_directories_in_the_diff(stencil_plan):
            stencil_plan.stdout_contains("--- old/tests    (directory not found)")
            stencil_plan.stdout_contains("+++ new/tests    (new directory)")

    def describe_applying_changes():

        @pytest.fixture(scope="module")
        def stencil_apply(stencil_init):
            yield (Stencil.apply()
                .dest(stencil_init.dest)
                .override("project.src", STENCIL_OVERRIDE_PATH)
                .run())

        def it_returns_success(stencil_apply):
            (stencil_apply
                .returncode(0)
                .stdout_contains(f"Applying changes from {STENCIL_OVERRIDE_PATH} to {stencil_apply.dest}"))

        def it_contains_new_file_in_the_diff(stencil_apply):
            stencil_apply.stdout_contains("--- old/my_project/__version__.py    (file not found)")
            stencil_apply.stdout_contains("+++ new/my_project/__version__.py")

        def it_does_not_contain_unchanged_files_in_the_diff(stencil_apply):
            stencil_apply.not_stdout_contains("pyproject.toml")

        def it_contains_file_updates_in_the_diff(stencil_apply):
            stencil_apply.stdout_contains("--- old/README.md")
            stencil_apply.stdout_contains("+++ new/README.md")
            stencil_apply.stdout_contains("-   4 B")
            stencil_apply.stdout_contains("+   4 X")

        def it_contains_new_directories_in_the_diff(stencil_apply):
            stencil_apply.stdout_contains("--- old/tests    (directory not found)")
            stencil_apply.stdout_contains("+++ new/tests    (new directory)")

        def it_creates_and_updates_files(stencil_init):
            files = slurp(Path(stencil_init.dest))
            expected_files = {
                ".stencil.toml": f'[stencil]\nversion = "1"\n\n[project]\nname = "my_project"\nsrc = "{STENCIL_PATH}"\n\n[arguments]\n',
                "README.md": "# my_project Documentation\n\nA\nX\nC\n",
                "pyproject.toml": "[project]\nname = my_project\n",
                ".github/CODEOWNERS": "* @all_the_engineers\n* @all_the_managers\n",
                "my_project/__init__.py": "",
                "my_project/__version__.py": '__version__ = "TODO: your version here"\n',
                "tests/__init__.py": "",
                "tests/test_example.py": "# some example tests\ndef test():\n    assert True\n"
            }
            assert files == expected_files


@pytest.mark.github
def describe_github_stencil():

    STENCIL_PATH = "gh://dstanek/stencil-test"

    @pytest.fixture(scope="module")
    def stencil_init(tmp_path_factory):
        tmp_path = tmp_path_factory.mktemp("output") / "output"
        yield (Stencil.init()
               .dest(tmp_path)
               .src(STENCIL_PATH)
               .run())

    def describe_successful_init():

        def it_returns_success(stencil_init):
            (stencil_init
                .returncode(0)
                .stdout_contains(f"Successfully initialized {stencil_init.dest}"))

        def it_creates_directory(stencil_init):
            assert stencil_init.dest.exists()

        def it_creates_files(stencil_init):
            files = slurp(stencil_init.dest)
            expected_files = {
                ".github/CODEOWNERS": "* @all_the_engineers\n* @all_the_managers",
                ".stencil.toml": f'[stencil]\nversion = "1"\n\n[project]\nname = "my_project"\nsrc = "{STENCIL_PATH}"\n\n[arguments]\n',
                "README.md": "# my_project Documentation\n\nX\nY\nZ\n",
                "pyproject.toml": "[project]\nname = my_project",
                "my_project/__init__.py": "",
                "my_project/__version__.py": "__version__ = \"TODO: your version here\"",
                "tests/__init__.py": "",
                "tests/test_example.py": "# some example tests\ndef test():\n    assert True"
            }
            assert files == expected_files

        def it_contains_new_directories_in_the_diff(stencil_init):
            stencil_init.stdout_contains("--- old/my_project    (directory not found)")
            stencil_init.stdout_contains("+++ new/my_project    (new directory)")

        def it_contains_new_empty_file_in_the_diff(stencil_init):
            stencil_init.stdout_contains("--- old/my_project/__init__.py    (file not found)")
            stencil_init.stdout_contains("+++ new/my_project/__init__.py    (new empty file)")

        def it_contains_new_file_in_the_diff(stencil_init):
            stencil_init.stdout_contains("--- old/pyproject.toml    (file not found)")
            stencil_init.stdout_contains("+++ new/pyproject.toml")


@pytest.mark.github
def describe_github_stencil_using_alternative_path():

    STENCIL_PATH = "gh://dstanek/stencil-test/other_stencil"

    @pytest.fixture(scope="module")
    def stencil_init(tmp_path_factory):
        tmp_path = tmp_path_factory.mktemp("output") / "output"
        yield (Stencil.init()
               .dest(tmp_path)
               .src(STENCIL_PATH)
               .run())

    def describe_successful_init():

        def it_returns_success(stencil_init):
            (stencil_init
                .returncode(0)
                .stdout_contains(f"Successfully initialized {stencil_init.dest}"))

        def it_creates_directory(stencil_init):
            assert stencil_init.dest.exists()

        def it_creates_files(stencil_init):
            files = slurp(stencil_init.dest)
            expected_files = {
                ".github/CODEOWNERS": "* @all_the_engineers\n* @all_the_managers",
                ".stencil.toml": f'[stencil]\nversion = "1"\n\n[project]\nname = "my_project"\nsrc = "{STENCIL_PATH}"\n\n[arguments]\n',
                "README.md": "# my_project Other Documentation\n\nA\nB\nC\n",
                "pyproject.toml": "[project]\nname = my_project",
                "my_project-other/__init__.py": "",
                "my_project-other/__version__.py": "__version__ = \"TODO: your version here\"",
                "tests/__init__.py": "",
                "tests/test_example.py": "# some example tests\ndef test():\n    assert True"
            }
            assert files == expected_files

        def it_contains_new_directories_in_the_diff(stencil_init):
            stencil_init.stdout_contains("--- old/my_project-other    (directory not found)")
            stencil_init.stdout_contains("+++ new/my_project-other    (new directory)")

        def it_contains_new_empty_file_in_the_diff(stencil_init):
            stencil_init.stdout_contains("--- old/my_project-other/__init__.py    (file not found)")
            stencil_init.stdout_contains("+++ new/my_project-other/__init__.py    (new empty file)")

        def it_contains_new_file_in_the_diff(stencil_init):
            stencil_init.stdout_contains("--- old/pyproject.toml    (file not found)")
            stencil_init.stdout_contains("+++ new/pyproject.toml")

# Usecases:
  # directory changes show up as new directories
  # stencil does not keep track of things it needs to delete!

# def describe_github_stencil_using_incorrect_path():
#     pass

# Test for a GitHub 404 on init, plan and apply

# Plan and apply before nit

# Init on an existing directory

# Better error for loading a config tha doesn't exist

# It would be nice to allow init to be used on an existing directory

# Allow for some things to be ingored, like a copyright...
