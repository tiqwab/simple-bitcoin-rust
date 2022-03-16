import { NavDropdown as BNavDropdown } from "react-bootstrap";
import { LinkContainer as BLinkContainer } from "react-router-bootstrap";

function NavbarMenu() {
    return (
        <BNavDropdown title="Menu" id="navbar-menu">
            <BLinkContainer to="/show-my-address">
                <BNavDropdown.Item>Show my address</BNavDropdown.Item>
            </BLinkContainer>
            <BNavDropdown.Item disabled={true}>Load my Keys</BNavDropdown.Item>
            <BLinkContainer to="/update-blockchain">
                <BNavDropdown.Item>Update Blockchain</BNavDropdown.Item>
            </BLinkContainer>
            <BNavDropdown.Item disabled={true}>Quit</BNavDropdown.Item>
        </BNavDropdown>
    )
}

export default NavbarMenu;