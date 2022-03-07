import {NavDropdown as BNavDropdown} from "react-bootstrap";

function NavbarMenu() {
    return (
        <BNavDropdown title="Menu" id="navbar-menu">
            <BNavDropdown.Item href="#menu-address">Show my address</BNavDropdown.Item>
            <BNavDropdown.Item href="#menu-keys">Load my Keys</BNavDropdown.Item>
            <BNavDropdown.Item href="#menu-blockchain">Update Blockchain</BNavDropdown.Item>
            <BNavDropdown.Item href="#menu-quit">Quit</BNavDropdown.Item>
        </BNavDropdown>
    )
}

export default NavbarMenu;