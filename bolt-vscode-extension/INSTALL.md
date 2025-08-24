# Installing the Bolt VS Code Extension

## Quick Installation

### Option 1: Manual Installation

1. **Copy the extension folder** to your VS Code extensions directory:

   **Windows:**
   ```cmd
   copy bolt-vscode-extension %USERPROFILE%\.vscode\extensions\bolt-language-0.1.0\
   ```

   **macOS/Linux:**
   ```bash
   cp -r bolt-vscode-extension ~/.vscode/extensions/bolt-language-0.1.0/
   ```

2. **Reload VS Code** (Ctrl+Shift+P → "Developer: Reload Window")

3. **Open a .bolt file** to see syntax highlighting in action!

### Option 2: From VS Code

1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Click the "..." menu → "Install from VSIX..."
4. Navigate to the `bolt-vscode-extension` folder
5. Select the extension and install

## Testing the Extension

1. Create a new file with `.bolt` extension
2. Copy this test code:

```bolt
import { print } from "bolt:stdio"

fun test(x: Integer): ^Integer {
    val ptr := &x
    return ptr
}

val number := 42
val result := test(number)
print(result^)
```

3. You should see:
   - **Blue**: Keywords like `import`, `fun`, `val`
   - **Green**: Strings like `"bolt:stdio"`
   - **Purple**: Types like `Integer`
   - **Red/Orange**: Operators like `:=`, `&`, `^`
   - **Yellow**: Function names and numbers

## Uninstallation

Delete the extension folder:
- **Windows**: `%USERPROFILE%\.vscode\extensions\bolt-language-0.1.0`
- **macOS/Linux**: `~/.vscode/extensions/bolt-language-0.1.0`

Then reload VS Code.