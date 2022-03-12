import { NavDropdown as BNavDropdown } from "react-bootstrap";
import { LinkContainer as BLinkContainer } from "react-router-bootstrap";

function NavbarMenu() {
    return (
        <BNavDropdown title="Menu" id="navbar-menu">
            <BNavDropdown.Item href="#menu-address">Show my address</BNavDropdown.Item>
            <BNavDropdown.Item href="#menu-keys">Load my Keys</BNavDropdown.Item>
            <BLinkContainer to="/update-blockchain">
                <BNavDropdown.Item>Update Blockchain</BNavDropdown.Item>
            </BLinkContainer>
            <BNavDropdown.Item href="#menu-quit">Quit</BNavDropdown.Item>
        </BNavDropdown>
    )
}

export default NavbarMenu;