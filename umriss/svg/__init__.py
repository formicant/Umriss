from typing import Any, Callable, TypeVar
from umriss.contour import LineContour

from umriss.contour import Contour, CubicContour
from umriss.drawing import Drawing, LineDrawing, CubicDrawing
from .debug_colors import get_debug_color
from .element import Element
from .path_data import PathData


class SvgDocument:
    """
    Generates a simple SVG document with some groups of paths inside.
    
    Create an `SvgDocument`, add some drawings, then, `render` or `save` it.
    Drawings can be linear, quadratic, or cubic.
    All the coordinates are rounded to the specified number of `decimals`.
    """
    
    def __init__(self, width: float, height: float, decimals: int=2):
        self.decimals = decimals
        self.svg = Element('svg', width=width, height=height, xmlns=_xmlns, xmlns__xlink=_xmlns_xlink)
    
    
    def render(self) -> str:
        return _xml_declaration + self.svg.render()
    
    
    def save(self, filename: str) -> None:
        with open(filename, 'w') as file:
            file.write(self.render())
    
    
    def add_line_drawing(self, drawing: LineDrawing, scale: float=1.0) -> None:
        self._add_drawing(self._add_line_contour, drawing, scale)
    
    
    def add_cubic_drawing(self, drawing: CubicDrawing, scale: float=1.0) -> None:
        self._add_drawing(self._add_cubic_contour, drawing, scale)
    
    
    _TContour = TypeVar('_TContour', bound=Contour)
    
    def _add_drawing(self,
            add_contour: Callable[[PathData, _TContour, float], None],
            drawing: Drawing[Any],
            scale: float,
    ) -> None:
        attributes = dict()
        if scale != 1.0:
            attributes['transform'] = f'scale({1 / scale})'
        
        if len(drawing.references) > 0:
            defs = Element('defs')
            for index, glyph in enumerate(drawing.referenced_glyphs):
                path_data = PathData(self.decimals)
                for contour in glyph.contours:
                    add_contour(path_data, contour, scale)
                defs.add_child(Element('path',
                    id=f'g{index}',
                    d=path_data,
                    fill=get_debug_color(index)  # for debugging purposes
                ))
            self.svg.add_child(defs)
        
        group = Element('g', **attributes)
        for glyph in drawing.glyphs:
            path_data = PathData(self.decimals)
            for contour in glyph.contours:
                add_contour(path_data, contour, scale)
            group.add_child(Element('path', d=path_data))
        
        for ref in drawing.references:
            group.add_child(Element('use',
                xlink__href=f'#g{ref.index}',
                x=self._format_value(ref.offset[0]),
                y=self._format_value(ref.offset[1])
            ))
        
        self.svg.add_child(group)
    
    
    def _add_line_contour(self, path_data: PathData, contour: LineContour, scale: float) -> None:
        points = contour.points if scale == 1.0 else scale * contour.points
        
        start_point = points[0]
        path_data.add_move_node(start_point)
        
        for point in points[1:]:
            path_data.add_line_node(point)
        
        path_data.add_close_node()
    
    
    def _add_cubic_contour(self, path_data: PathData, contour: CubicContour, scale: float) -> None:
        nodes = contour.nodes if scale == 1.0 else scale * contour.nodes
        
        [_, _, end_point] = nodes[-1]
        path_data.add_move_node(end_point)
        
        for node in nodes:
            path_data.add_cubic_node(node)
        
        path_data.add_close_node()
    
    
    def _format_value(self, value: float) -> str:
        formatted = '{:.{d}f}'.format(value, d=self.decimals)
        if self.decimals > 0:
            return formatted.rstrip('0').rstrip('.')
        else:
            return formatted


_xml_declaration = '<?xml version="1.0" encoding="UTF-8"?>\n'

_xmlns = "http://www.w3.org/2000/svg"
_xmlns_xlink = "http://www.w3.org/1999/xlink"
