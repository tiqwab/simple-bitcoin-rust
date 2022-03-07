import {NavDropdown as BNavDropdown} from "react-bootstrap";

function NavbarSettings() {
    return (
        <BNavDropdown title="Settings" id="navbar-settings">
            <BNavDropdown.Item href="#settings-keys">Renew my keys</BNavDropdown.Item>
            <BNavDropdown.Item href="#settings-connection">Connection info</BNavDropdown.Item>
        </BNavDropdown>
    )
}

export default NavbarSettings;
