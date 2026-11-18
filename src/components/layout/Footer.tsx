import { APP_NAME } from "../../constants/site";
import { VERSION_APP } from "../../constants/site";

export function Footer() {
    return(
        <footer className="app-footer">
            <span>{APP_NAME}</span>
            <span>v{VERSION_APP} · Local mode</span>
        </footer>
    );
}