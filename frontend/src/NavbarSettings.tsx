import {NavDropdown as BNavDropdown} from "react-bootstrap";

function NavbarSettings() {
    return (
        <BNavDropdown title="Settings" id="navbar-settings">
            <BNavDropdown.Item disabled={true}>Renew my keys</BNavDropdown.Item>
            <BNavDropdown.Item disabled={true}>Connection info</BNavDropdown.Item>
        </BNavDropdown>
    )
}

export default NavbarSettings;
