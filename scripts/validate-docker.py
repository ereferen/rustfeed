#!/usr/bin/env python3
"""
Docker設定ファイルの検証スクリプト
"""
import sys
import re
from pathlib import Path

def validate_dockerfile(dockerfile_path):
    """Dockerfileの基本的な構文チェック"""
    print("📋 Dockerfile を検証中...")

    if not dockerfile_path.exists():
        print(f"❌ Dockerfileが見つかりません: {dockerfile_path}")
        return False

    content = dockerfile_path.read_text()
    errors = []
    warnings = []

    # FROM命令のチェック
    if not re.search(r'^FROM\s+\S+', content, re.MULTILINE):
        errors.append("FROM命令が見つかりません")

    # RUN, COPY, ADDなどの基本命令の存在確認
    if 'RUN' not in content:
        warnings.append("RUN命令がありません")

    # WORKDIRの存在確認
    if 'WORKDIR' not in content:
        warnings.append("WORKDIRが設定されていません")

    # エラー・警告の表示
    if errors:
        print("❌ エラー:")
        for error in errors:
            print(f"  - {error}")
        return False

    if warnings:
        print("⚠️  警告:")
        for warning in warnings:
            print(f"  - {warning}")

    print("✅ Dockerfileの検証完了")
    return True


def validate_docker_compose(compose_path):
    """docker-compose.ymlの基本的な構文チェック"""
    print("\n📋 docker-compose.yml を検証中...")

    if not compose_path.exists():
        print(f"❌ docker-compose.ymlが見つかりません: {compose_path}")
        return False

    content = compose_path.read_text()
    errors = []
    warnings = []

    # version指定のチェック
    if not re.search(r'^version:\s*["\']?\d+', content, re.MULTILINE):
        warnings.append("versionが指定されていません")

    # servicesセクションのチェック
    if 'services:' not in content:
        errors.append("servicesセクションが見つかりません")

    # volumesの使用チェック
    if 'volumes:' not in content:
        warnings.append("volumesが定義されていません（データが永続化されない可能性）")

    # エラー・警告の表示
    if errors:
        print("❌ エラー:")
        for error in errors:
            print(f"  - {error}")
        return False

    if warnings:
        print("⚠️  警告:")
        for warning in warnings:
            print(f"  - {warning}")

    print("✅ docker-compose.ymlの検証完了")
    return True


def validate_env_example(env_example_path):
    """env.exampleファイルのチェック"""
    print("\n📋 .env.example を検証中...")

    if not env_example_path.exists():
        print(f"❌ .env.exampleが見つかりません: {env_example_path}")
        return False

    content = env_example_path.read_text()

    if 'ANTHROPIC_API_KEY' not in content:
        print("⚠️  ANTHROPIC_API_KEYの設定例がありません")

    print("✅ .env.exampleの検証完了")
    return True


def check_gitignore(gitignore_path):
    """gitignoreに.envが含まれているかチェック"""
    print("\n📋 .gitignore を検証中...")

    if not gitignore_path.exists():
        print("⚠️  .gitignoreが見つかりません")
        return True

    content = gitignore_path.read_text()

    if '.env' not in content:
        print("⚠️  .gitignoreに.envが含まれていません（APIキーが漏洩する可能性）")
        return False

    print("✅ .gitignoreの検証完了")
    return True


def main():
    print("=" * 60)
    print("Docker環境設定ファイルの検証")
    print("=" * 60)

    # プロジェクトルートの取得
    script_dir = Path(__file__).parent
    project_root = script_dir.parent

    # 各ファイルの検証
    results = []
    results.append(validate_dockerfile(project_root / "Dockerfile"))
    results.append(validate_docker_compose(project_root / "docker-compose.yml"))
    results.append(validate_env_example(project_root / ".env.example"))
    results.append(check_gitignore(project_root / ".gitignore"))

    print("\n" + "=" * 60)
    if all(results):
        print("✅ すべての検証に成功しました！")
        print("\n次のステップ:")
        print("1. .envファイルを作成: cp .env.example .env")
        print("2. .envにAPI Keyを設定")
        print("3. Docker環境を起動: make setup && make up")
        print("=" * 60)
        return 0
    else:
        print("❌ 一部の検証に失敗しました")
        print("=" * 60)
        return 1


if __name__ == "__main__":
    sys.exit(main())
