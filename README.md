# UniFi Echo

> 🍃 Rooted in nature 🍁, 🌱 branching into decentralized finance — 🧑🏻‍💻 **echoing** ideas 💡 into a 🌍 borderless financial world.

<p align="left">
  <img src="res/echo_bg.png" alt="Echo" width="50%" height="auto">
</p>

## API

### 🔗 **API Configuration Guide**

See full API reference [here](./api/README.md).

### ⚙️ Setup REST Client Environment

To simplify API testing in VS Code using the [REST Client extension](https://marketplace.visualstudio.com/items?itemName=humao.rest-client), create a workspace settings file:

> [!NOTE]
> Ensure the VSCode extension is installed in your VSCode editor.

File path:

```sh
.vscode/settings.json
```

#### Sample configuration

```json
{
    "rest-client.environmentVariables": {
        "prod": {
            "base_url": "https://unifi-api-jlq9.onrender.com",
            "api_key": "YOUR_API_KEY"
        }
    }
}
```

Replace `YOUR_API_KEY` with your one. Refer [this](./api/README.md#-get-your-api-key)

#### 🧭 Selecting the Environment

1. Open the Command Palette:
   - macOS: <kbd>Cmd + Shift + P</kbd>
   - Windows/Linux: <kbd>Ctrl + Shift + P</kbd>
2. Type and select **“Rest Client: Switch Environment”**.
3. Choose the environment — e.g., **prod**.

#### 📦 Using the Variables

After selecting the environment, you can directly reference the variables inside your .http files:

```http
{{base_url}}
{{api_key}}
```

Example:

```http
GET {{base_url}}/v1/payments
Authorization: Bearer {{api_key}}
```
